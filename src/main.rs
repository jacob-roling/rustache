use std::{
    io::{BufReader, Cursor},
    thread,
};

use rustache::{lexer::lex, parser::parse};

fn main() {
    // println!("{:#?}", templates("views", "**/*.mustache"));
    // let file = File::open("test.mustache").expect("Failed to open file");
    // let reader = BufReader::with_capacity(64, file);

    let (token_sender, token_reciever) = crossbeam_channel::bounded(2);

    thread::spawn(move || {
        let input = String::from("{{default_tags}}{{=<% %>=}}<%new_tags%>");
        let reader = BufReader::with_capacity(128, Cursor::new(input));
        lex(reader, token_sender);
    });

    let template_handle = thread::spawn(move || {
        return parse(token_reciever);
    });

    match template_handle.join() {
        Ok(result) => match result {
            Ok(node) => {
                println!("{:#?}", node);
            }
            Err(e) => {
                println!("{:#?}", e);
            }
        },
        Err(e) => {
            println!("{:#?}", e);
        }
    }
}
