use serde::{Deserialize, Serialize};
use rustache::node::Value;
use glob::glob;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Spec {
    overview: String,
    tests: Vec<Test>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Test {
    name: String,
    desc: String,
    template: String,
    expected: String
}

// #[test]
// fn spec() {
//     for entry in glob("spec/specs/*.json").expect("failed to read glob pattern") {
//         if let Ok(path) = entry {
//             let contents = fs::read_to_string(path).expect("failed to read file");
//             let spec: Spec = serde_json::from_str(&contents).expect("failed to parse spec file");
//             println!("{:#?}", spec);
//         }
//     }
//     // println!("{:#?}", spec);
// }

#[test]
fn inverted() {
    let contents = fs::read_to_string("spec/spec/inverted.json").expect("failed to read file");
    let spec: Spec = serde_json::from_str(&contents).expect("failed to parse spec file");
    println!("{:#?}", spec);
}