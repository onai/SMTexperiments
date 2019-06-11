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

extern crate serde_json;
extern crate z3_sched;

use z3_sched::{build_schedule, AllOf, Commit};

use serde_json::{Error, Value};
use std::collections::HashMap;
use std::env;
use std::fs::File;

fn main() {
    let arguments: Vec<String> = env::args().collect();

    if arguments.len() < 2 {
        eprintln!("Usage: /path/to/json/file.json");
        std::process::exit(1);
    }

    let path_to_system = &arguments[1];

    build_allofs(path_to_system.to_string());
}

fn build_allofs(path: String) {
    let mut f = File::open(path).expect("File not found");
    let v: Value = serde_json::from_reader(f).expect("Expected json payload in file");

    // this is an array of allofs
    let commits_values = v.as_array().unwrap();

    let mut commits = Vec::new();

    for commit_value in commits_values {
        let mut allofs = Vec::new();
        let allofs_values = commit_value.as_array().unwrap();

        for allof_value in allofs_values {
            let s_calls = allof_value.as_object().unwrap().get("s_calls").unwrap();
            let mut service_instances = HashMap::new();
            let cost_ceil = allof_value
                .as_object()
                .unwrap()
                .get("cost_ceil")
                .unwrap()
                .as_i64()
                .unwrap();
            for s_call_entry in s_calls.as_array().unwrap() {
                let s_call_arr = s_call_entry.as_array().unwrap();
                let s_call = s_call_arr[0].as_str().unwrap().to_string();
                let is_request = s_call_arr[1].as_bool().unwrap();
                service_instances.insert(s_call.to_string(), is_request);
            }

            allofs.push(AllOf {
                service_instances: service_instances,
                cost_ceil: cost_ceil,
            })
        }

        commits.push(Commit { allofs: allofs });

        if commits.len() > 10 {
            break;
        }
    }

    build_schedule(commits);
}
