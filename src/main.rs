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

    if let Some(error) = rustache.render(
        "index",
        &mut stdout,
        &Index {
            greeting: "Test".into(),
        },
    ) {
        println!("{}", error);
    }

    stdout.flush().unwrap();
}
