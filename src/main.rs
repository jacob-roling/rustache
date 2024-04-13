use std::{
    fs::File,
    io::{BufReader, Cursor},
    sync::mpsc,
};

use rustache::{
    lexer::{lex, Token},
    templates,
};

fn main() {
    println!("{:#?}", templates("views", "**/*.mustache"));
    // let input = String::from("asdas");
    // let reader = BufReader::with_capacity(64, Cursor::new(input));
    // let file = File::open("test.mustache").expect("Failed to open file");
    // let reader = BufReader::with_capacity(64, file);
    // let (sender, reciever) = mpsc::sync_channel::<Token>(2);
    // lex(reader, sender);

    // while let Ok(token) = reciever.recv() {
    //     println!("{:#?}", token);
    // }
}
