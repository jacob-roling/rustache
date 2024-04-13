use std::sync::mpsc::Receiver;

use crate::lexer::Token;

pub struct Parser {
    input: Receiver<Token>,
}

impl Parser {
    pub fn new(input: Receiver<Token>) -> Self {
        return Self { input };
    }

    pub fn next(&self) -> Token {
        return match self.input.recv() {
            Ok(lexeme) => lexeme,
            Err(e) => panic!("Receiving lexeme: {}", e),
        };
    }
}
