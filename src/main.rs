use std::{
    io::{BufReader, Cursor},
    thread,
};

use rustache::lexer::lex;

fn main() {
    // println!("{:#?}", templates("views", "**/*.mustache"));
    // let file = File::open("test.mustache").expect("Failed to open file");
    // let reader = BufReader::with_capacity(64, file);
    let (sender, reciever) = crossbeam_channel::bounded(2);

    thread::spawn(move || {
        let input = String::from("{{default_tags}}{{=<% %>=}}<%new_tags%>");
        let reader = BufReader::with_capacity(128, Cursor::new(input));
        lex(reader, sender);
    });

    while let Ok(token) = reciever.recv() {
        println!("{:#?}", token);
    }
}
