use rustache::{EmptyContext, Rustache};
use serde::Serialize;
use std::io::Write;

#[derive(Debug, Serialize)]
struct Index {
    greeting: String,
}

fn main() {
    let rustache = Rustache::new("views", "**/*.mustache").unwrap();
    let mut stdout = std::io::stdout();

    // println!("{:#?}", rustache);

    if let Some(error) = rustache.render("index", &mut stdout, &EmptyContext) {
        println!("{}", error);
    }

    // if let Some(error) = rustache.render(
    //     "fruit",
    //     &mut stdout,
    //     Some(&Value::Object(HashMap::from([(
    //         "fruit".into(),
    //         Value::Array(Vec::from([
    //             Value::String("apple".into()),
    //             Value::String("banana".into()),
    //             Value::String("pear".into()),
    //         ])),
    //     )]))),
    // ) {
    //     println!("{}", error);
    // }

    stdout.flush().unwrap();
}
