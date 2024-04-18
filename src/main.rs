use std::{collections::HashMap, io::Write};

use rustache::{node::Value, Rustache};

fn main() {
    let rustache = Rustache::new("views", "**/*.mustache").unwrap();
    let mut stdout = std::io::stdout().lock();

    // println!("{:#?}", rustache);

    if let Some(error) = rustache.render(
        "test",
        &mut stdout,
        Some(&Value::Object(HashMap::from([(
            "greeting".into(),
            Value::String("world".into()),
        )]))),
    ) {
        println!("{}", error);
    }

    stdout.flush().unwrap();
}
