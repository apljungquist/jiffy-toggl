use itertools::Itertools;
use serde_json::{Map, Value};
use std::env;
use std::fs::File;
use std::io::BufReader;


fn flattened(root: Map<String, Value>) -> Vec<(String, Value)> {
    let mut output = Vec::new();
    let mut values = vec![(String::new(), Value::Object(root))];
    while let Some((path, value)) = values.pop() {
        match value {
            Value::Null => output.push((path, Value::Null)),
            Value::Bool(b) => output.push((path, Value::Bool(b))),
            Value::Number(n) => output.push((path, Value::Number(n))),
            Value::String(s) => output.push((path, Value::String(s.clone()))),
            Value::Array(a) => {
                for v in a.into_iter() {
                    let mut p = path.clone();
                    if !p.is_empty() {
                        p.push('.');
                    }
                    values.push((p, v));
                }
            }
            Value::Object(o) => {
                for (k, v) in o.into_iter().rev() {
                    debug_assert!(!k.contains('.'));
                    let mut p = path.clone();
                    if !p.is_empty() {
                        p.push('.');
                    }
                    p.push_str(&k.to_string());
                    values.push((p, v));
                }
            }
        }
    }
    output
}

fn analyze(path: &str) {
    let data: Value = serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap();
    let Value::Object(o) = data else {
        panic!("Not an object");
    };
    flattened(o)
        .into_iter()
        .into_group_map()
        .into_iter()
        .for_each(|(k, vs)| {
            println!("{k}");
            let cs = vs
                .into_iter()
                .counts()
                .into_iter()
                .sorted_by_key(|(_, c)| std::cmp::Reverse(*c))
                .collect_vec();
            for (v, c) in cs.iter().take(10) {
                println!("  {v}: {c}");
            }
            let remaining = cs.len().saturating_sub(10);
            if 0 < remaining {
                println!("  and {remaining} more");
            }
        })
}

fn main() {
    env_logger::init();
    analyze(env::var("JIFFY2TOGGLE_BACKUP_FILE").unwrap().as_str());
}