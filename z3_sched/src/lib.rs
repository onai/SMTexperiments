//  Copyright 2018- Onai (Onu Technology, Inc., San Jose, California)
//
//  Permission is hereby granted, free of charge, to any person obtaining a copy
//  of this software and associated documentation files (the "Software"), to deal
//  in the Software without restriction, including without limitation the rights
//  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//  copies of the Software, and to permit persons to whom the Software is furnished
//  to do so, subject to the following conditions:

//  The above copyright notice and this permission notice shall be included in all
//   copies or substantial portions of the Software.

//  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
//   INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
//  PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
//  HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
//  OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
//  SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

extern crate z3;

use std::collections::HashMap;
use z3::*;

#[derive(Debug)]
pub struct Commit {
    pub allofs: Vec<AllOf>,
}

#[derive(Debug)]
pub struct AllOf {
    pub service_instances: HashMap<String, bool>,
    pub cost_ceil: i64,
}

/// build vars
pub fn build_schedule(commits: Vec<Commit>) {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // contains entries of the form:
    //  [
    //    '0': Bool('0')
    //  ]
    let commit_bools = build_commit_level_vars(&ctx, &commits);

    // contains entries of the form:
    // [
    //   '0-0': Bool('0-0') - commit 0, allof 0
    // ]
    let allof_bools = build_allof_level_bools(&ctx, &commits);

    // contains entries of the form:
    // [
    //   '0-0-abcde-0': Bool('0-0-abce-0') - commit 0, allof 0, service_call_regid abcde, instance 0
    // ]
    let s_call_bools = build_scall_level_bools(&ctx, &commits);

    // first group service-call names by their allof
    let s_calls_grouped = group_s_calls(&s_call_bools);

    // grab count vars
    let s_call_prices = build_count_vars(&ctx, &commits);

    // group allofs by the commitment they are part of
    let allofs_grouped = group_allofs(&allof_bools);

    println!("{:#?}", s_call_bools.keys());

    println!("{:#?}", s_calls_grouped);
    println!("{:#?}", allofs_grouped);

    let solver = Optimize::new(&ctx);

    // a commitment is scheduled implies that exactly one of its allofs is scheduled
    for (commit_var_name, commit_var) in &commit_bools {
        let commit_allofs = &allofs_grouped[commit_var_name];

        let mut allof_vars = Vec::new();
        for var_name in commit_allofs {
            allof_vars.push(&allof_bools[var_name]);
        }
        let mut allof_coeffs = vec![];
        for _ in commit_allofs {
            allof_coeffs.push(1);
        }
        let allof_count_clause =
            allof_vars[0].pb_eq(&allof_vars[1..allof_vars.len()], allof_coeffs, 1);

        solver.assert(&commit_var.implies(&allof_count_clause));
    }

    println!("commitment one_of clause built");

    // if a commitment is not scheduled then none of the included allofs should be scheduled
    for (commit_var_name, commit_var) in &commit_bools {
        let commit_allofs = &allofs_grouped[commit_var_name];

        let mut allof_vars = Vec::new();
        for var_name in commit_allofs {
            allof_vars.push(&allof_bools[var_name]);
        }
        let mut allof_coeffs = vec![];
        for _ in commit_allofs {
            allof_coeffs.push(1);
        }
        let allof_count_clause2 =
            allof_vars[0].pb_eq(&allof_vars[1..allof_vars.len()], allof_coeffs, 0);

        solver.assert(&commit_var.not().implies(&allof_count_clause2));
    }

    println!("commitment one_of negation clause built");

    // if a particular allof is scheduled then we need to schedule all its service
    // calls
    for (allofs_var_name, allof_var) in &allof_bools {
        println!("allof var name {}", allofs_var_name);
        let allof_s_calls = &s_calls_grouped[allofs_var_name];

        println!("Contains s_calls: {:?}", allof_s_calls);

        let mut s_call_vars = Vec::new();
        for var_name in allof_s_calls {
            s_call_vars.push(&s_call_bools[var_name].0);
        }

        let allof_scall_clause = s_call_vars[0].and(&s_call_vars[1..s_call_vars.len()]);

        solver.assert(&allof_var.implies(&allof_scall_clause));
    }

    println!("allof service_call clause built");

    // if an allof is not scheduled, none of the constituent s_calls must be scheduled
    for (allofs_var_name, allof_var) in &allof_bools {
        let allof_s_calls = &s_calls_grouped[allofs_var_name];

        let mut s_call_vars = Vec::new();
        for var_name in allof_s_calls {
            s_call_vars.push(&s_call_bools[var_name].0);
        }

        let mut coeffs = Vec::new();
        for _ in allof_s_calls {
            coeffs.push(1);
        }

        let allof_scall_clause2 =
            s_call_vars[0].pb_eq(&s_call_vars[1..s_call_vars.len()], coeffs, 0);

        solver.assert(&allof_var.not().implies(&allof_scall_clause2));
    }

    println!("allof service_call negation clause built");

    // request and offer clauses

    // group by requests and offers
    let mut request_s_calls = HashMap::new();
    let mut offer_s_calls = HashMap::new();

    for (s_call_varname, (s_call_var, is_request)) in &s_call_bools {
        let splits: Vec<&str> = s_call_varname.split("-").collect();
        let mut s_call_portion = Vec::new();

        for s in splits[2..splits.len()].iter() {
            s_call_portion.push(s.to_string());
        }

        let s_call_instance_id_bits: String = s_call_portion.join("-").into();

        if *is_request {
            let entry = request_s_calls
                .entry(s_call_instance_id_bits.clone())
                .or_insert(Vec::new());
            entry.push(s_call_var);
        } else {
            let entry = offer_s_calls
                .entry(s_call_instance_id_bits.clone())
                .or_insert(Vec::new());
            entry.push(s_call_var);
        }
    }

    // if a request has been scheduled, exactly one offer must be scheduled for it
    for (s_call_instance, s_call_var) in &request_s_calls {
        println!("handling Request: {}", s_call_instance);
        let maybe_offer_scall_vars = &offer_s_calls.get(s_call_instance);

        if maybe_offer_scall_vars.is_none() {
            // then this service call set can basically not be scheduled
            println!("Found no offer");
            for var in s_call_var {
                solver.assert(&var.not());
            }
        } else {
            println!("Found an offer");
            let offer_scall_vars = maybe_offer_scall_vars.unwrap();

            let mut req_var_refs = Vec::new();
            for x in s_call_var {
                req_var_refs.push(*x);
            }
            let request_clause = req_var_refs[0].or(&req_var_refs[1..req_var_refs.len()]);

            let mut offer_var_refs = Vec::new();
            for x in offer_scall_vars {
                offer_var_refs.push(*x);
            }

            // clause needed if multiple offers exist for this one request
            if offer_s_calls.len() > 1 {
                let mut coeffs = Vec::<i32>::new();
                for _ in offer_scall_vars {
                    coeffs.push(1);
                }

                let one_offer_clause =
                    offer_var_refs[0].pb_eq(&offer_var_refs[1..offer_scall_vars.len()], coeffs, 1);

                solver.assert(&request_clause.implies(&one_offer_clause));

                let mut coeffs2 = Vec::<i32>::new();
                for _ in offer_scall_vars {
                    coeffs2.push(1);
                }
            }
            // if only one offer exists however, use this clause
            else {
                solver.assert(&request_clause.implies(offer_var_refs[0]));
            }
        }
    }

    println!("only one offer clause built");

    // if a service call is being offered, then at least one request must exist for it
    for (s_call_instance, s_call_var) in offer_s_calls {
        let maybe_req_scall_vars = &request_s_calls.get(&s_call_instance);

        if maybe_req_scall_vars.is_none() {
            // do not schedule this service call
            for var in s_call_var {
                solver.assert(&var.not());
            }
        } else {
            let mut coeffs = Vec::<i32>::new();
            let req_scall_vars = maybe_req_scall_vars.unwrap();

            let mut req_var_refs = Vec::new();
            for x in req_scall_vars {
                req_var_refs.push(*x);
            }

            let mut offer_var_refs = Vec::new();
            for x in &s_call_var {
                offer_var_refs.push(*x);
            }

            let offer_clause = offer_var_refs[0].or(&offer_var_refs[1..offer_var_refs.len()]);

            if req_scall_vars.len() > 1 {
                for _ in req_scall_vars {
                    coeffs.push(1);
                }

                let at_least_one_request =
                    req_var_refs[0].pb_ge(&req_var_refs[1..req_var_refs.len()], coeffs, 1);

                solver.assert(&offer_clause.implies(&at_least_one_request));
            } else {
                // only one request so no need for a pb. must be scheduled
                solver.assert(&offer_clause.implies(req_var_refs[0]));
            }
        }
    }
    println!("at least one request clause built");

    // schedule must be non-trivial - at least one commit
    let mut commit_vars_list = Vec::new();
    for (var_name, commit_var) in &commit_bools {
        commit_vars_list.push(commit_var);
    }

    let mut coeffs = Vec::new();
    for _ in 0..commit_vars_list.len() {
        coeffs.push(1);
    }

    // make sure that at least one commitment is scheduled
    let at_least_one_commit =
        commit_vars_list[0].pb_ge(&commit_vars_list[1..commit_vars_list.len()], coeffs, 1);
    solver.assert(&at_least_one_commit);

    // make sure that the cost ceilings are respected
    println!("respectin cost ceilings");
    for (i, commit) in commits.iter().enumerate() {
        for (j, allof) in commit.allofs.iter().enumerate() {
            let cost_ceil = ctx.from_i64(allof.cost_ceil);
            let allof_name = format!("{}-{}", i, j);
            //let mut s_call_costs = Vec::new();

            let s_calls = s_calls_grouped.get(&allof_name).unwrap();
            let mut allof_costs = ctx.from_i64(0);

            for s_call_var_name in s_calls {
                let (_s_call_var, is_request) = s_call_bools.get(s_call_var_name).unwrap();
                let mut coeff;

                if *is_request {
                    coeff = ctx.from_i64(1);
                } else {
                    coeff = ctx.from_i64(-1);
                }

                let s_call_splits: Vec<&str> = s_call_var_name.split("-").collect();
                let s_call_portion: Vec<&str> = s_call_splits[2..s_call_splits.len() - 1].to_vec();
                let mut s_call_portion_strings = Vec::new();

                for s in s_call_portion.iter() {
                    s_call_portion_strings.push(s.to_string());
                }

                let s_call_str = s_call_portion_strings.join("-");

                let cost_var = s_call_prices.get(&s_call_str).unwrap();

                let cost_entry = cost_var.mul(&[&coeff]);
                allof_costs = allof_costs.add(&[&cost_entry]);
            }
            solver.assert(&allof_costs.le(&cost_ceil));
        }
    }

    // maximize commitments scheduled
    let mut n_commits = ctx.from_i64(0);
    for (i, commit) in commits.iter().enumerate() {
        let commit_varname = format!("{}", i);
        let commit_bool = commit_bools.get(&commit_varname).unwrap();

        n_commits = n_commits.add(&[&commit_bool.ite(&ctx.from_i64(1), &ctx.from_i64(0))]);
    }
    solver.maximize(&n_commits);

    println!("{:?}", solver.check());

    if solver.check() {
        let model = solver.get_model();
        for (name, var) in &commit_bools {
            println!("{}: {}", name, model.eval(var).unwrap().as_bool().unwrap());
        }

        for (name, var) in &allof_bools {
            println!("{}: {}", name, model.eval(var).unwrap().as_bool().unwrap());
        }

        for (name, var) in &s_call_bools {
            println!(
                "{}: {}",
                name,
                model.eval(&var.0).unwrap().as_bool().unwrap()
            );
        }

        for (name, var) in &s_call_prices {
            //println!("name: {}", name);
            println!("{}: {:?}", name, model.eval(&var).unwrap().as_real());
        }
    }
}

pub fn build_commit_level_vars<'ctx>(
    ctx: &'ctx Context,
    commits: &Vec<Commit>,
) -> HashMap<String, Ast<'ctx>> {
    let mut commit_bools = HashMap::new();

    for (i, commit) in commits.iter().enumerate() {
        let cur_commit_var_name = i.to_string();

        let commit_bool = ctx.named_bool_const(cur_commit_var_name.as_str());
        commit_bools.insert(cur_commit_var_name, commit_bool);
    }

    commit_bools
}

pub fn build_allof_level_bools<'ctx>(
    ctx: &'ctx Context,
    commits: &Vec<Commit>,
) -> HashMap<String, Ast<'ctx>> {
    let mut allof_bools = HashMap::new();

    for (i, commit) in commits.iter().enumerate() {
        for (j, allof) in commit.allofs.iter().enumerate() {
            let cur_allof_var_name = format!("{}-{}", i, j);

            let var = ctx.named_bool_const(cur_allof_var_name.as_str());
            allof_bools.insert(cur_allof_var_name, var);
        }
    }

    allof_bools
}

pub fn build_scall_level_bools<'ctx>(
    ctx: &'ctx Context,
    commits: &Vec<Commit>,
) -> HashMap<String, (Ast<'ctx>, bool)> {
    let mut scall_bools = HashMap::new();

    for (i, commit) in commits.iter().enumerate() {
        for (j, allof) in commit.allofs.iter().enumerate() {
            for (s_call_regid, is_request) in &allof.service_instances {
                let var_name = format!("{}-{}-{}", i, j, s_call_regid);

                let var = ctx.named_bool_const(var_name.as_str());
                scall_bools.insert(var_name, (var, *is_request));
            }
        }
    }

    scall_bools
}

/// Result:
/// Allof (name) -> service-call-var-name
pub fn group_s_calls<'ctx>(
    s_call_bools: &HashMap<String, (Ast<'ctx>, bool)>,
) -> HashMap<String, Vec<String>> {
    let mut name_group = HashMap::new();

    for (s_call_name, _) in s_call_bools {
        let splits: Vec<&str> = s_call_name.split("-").collect();

        let commit_id = splits[0];
        let allof_id = splits[1];

        let entry = name_group
            .entry(format!("{}-{}", commit_id, allof_id))
            .or_insert(Vec::new());
        entry.push(s_call_name.clone());
    }

    name_group
}

pub fn group_allofs<'ctx>(
    allof_bools: &HashMap<String, Ast<'ctx>>,
) -> HashMap<String, Vec<String>> {
    let mut name_group = HashMap::new();

    for (allof_name, _) in allof_bools {
        let splits: Vec<&str> = allof_name.split("-").collect();

        let commit_id = splits[0];

        let entry = name_group
            .entry(format!("{}", commit_id))
            .or_insert(Vec::new());

        entry.push(allof_name.clone())
    }

    name_group
}

/// Builds a per-service call cost integer
pub fn build_count_vars<'ctx>(
    ctx: &'ctx Context,
    commits: &Vec<Commit>,
) -> HashMap<String, Ast<'ctx>> {
    let mut s_call_costs = HashMap::new();

    for commit in commits {
        for allof in &commit.allofs {
            for (s_call_instance, is_request) in &allof.service_instances {
                // drop the instance id first
                let splits: Vec<&str> = s_call_instance.split("-").collect();
                let mut splits_str = Vec::new();
                let limit = splits.len() - 1;

                for (i, portion) in splits.into_iter().enumerate() {
                    if i < limit {
                        splits_str.push(portion.to_string());
                    }
                }

                let s_call_portion: String = splits_str.join("-").into();
                let s_call_price = ctx.named_int_const(s_call_portion.as_str());

                s_call_costs.insert(s_call_portion, s_call_price);
            }
        }
    }

    s_call_costs
}
