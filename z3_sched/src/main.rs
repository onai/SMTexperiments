extern crate serde_json;
extern crate z3_sched;

use std::collections::HashMap;
use z3_sched::{build_schedule, AllOf, Commit};

fn build_unsat1() -> Vec<Commit> {
    let mut allof0_sids = HashMap::new();

    allof0_sids.insert("abcde-0".into(), true);
    allof0_sids.insert("abcde-1".into(), true);

    let allof0 = AllOf {
        service_instances: allof0_sids,
        cost_ceil: 10,
    };

    let mut allof1_sids = HashMap::new();

    allof1_sids.insert("abcde-0".into(), false);

    let allof1 = AllOf {
        service_instances: allof1_sids,
        cost_ceil: 20,
    };

    let mut allof2_sids = HashMap::new();

    allof2_sids.insert("abcde-0".into(), false);

    let allof2 = AllOf {
        service_instances: allof2_sids,
        cost_ceil: 20,
    };

    let commit1 = Commit {
        allofs: vec![allof0],
    };
    let commit2 = Commit {
        allofs: vec![allof1],
    };
    let commit3 = Commit {
        allofs: vec![allof2],
    };

    vec![commit1, commit2, commit3]
}

fn build_sat1() -> Vec<Commit> {
    let mut allof0_sids = HashMap::new();
    allof0_sids.insert("abcde-0".into(), true);

    let mut allof1_sids = HashMap::new();
    allof1_sids.insert("efgh-0".into(), true);

    let allof0 = AllOf {
        service_instances: allof0_sids,
        cost_ceil: 100,
    };

    let allof1 = AllOf {
        service_instances: allof1_sids,
        cost_ceil: 100,
    };

    let commit1 = Commit {
        allofs: vec![allof0, allof1],
    };

    let mut allof2_sids = HashMap::new();
    allof2_sids.insert("abcde-0".into(), false);

    let allof2 = AllOf {
        service_instances: allof2_sids,
        cost_ceil: -50,
    };
    let commit2 = Commit {
        allofs: vec![allof2],
    };

    let mut allof3_sids = HashMap::new();
    allof3_sids.insert("abcde-0".into(), false);
    let allof3 = AllOf {
        service_instances: allof3_sids,
        cost_ceil: -50,
    };
    let commit3 = Commit {
        allofs: vec![allof3],
    };

    vec![commit1, commit2, commit3]
}

fn main() {
    let unsat1 = build_unsat1();
    println!("{:#?}", unsat1);
    build_schedule(unsat1);

    let sat1 = build_sat1();
    println!("{:#?}", sat1);
    build_schedule(sat1);
}
