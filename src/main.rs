use rustache::{node::Value, Context, Rustache};
use std::{collections::HashMap, io::Write};

#[derive(Debug, Context)]
pub struct Test {
    list: Vec<usize>,
    text: String,
    num: f32,
    map: Fruit,
}

#[derive(Debug, Context)]
struct Fruit {
    name: String,
}

fn main() {
    let test = Test {
        list: Vec::from([1, 2, 3]),
        text: "hi".into(),
        num: 16.,
        map: Fruit {
            name: "banana".into(),
        },
    };

    println!("{:#?}", test.to_context());

    // let rustache = Rustache::new("views", "**/*.mustache").unwrap();
    // let mut stdout = std::io::stdout().lock();

    // // println!("{:#?}", rustache);

    // if let Some(error) = rustache.render(
    //     "index",
    //     &mut stdout,
    //     Some(&Value::Object(HashMap::from([(
    //         "greeting".into(),
    //         Value::String("world".into()),
    //     )]))),
    // ) {
    //     println!("{}", error);
    // }

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

    // stdout.flush().unwrap();
}
