use anyhow::Result;
use crossbeam_channel::Receiver;
use thiserror::Error;

use crate::{lexer::Token, node::Node};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("test")]
    Test,
}

pub struct Parser {
    tokens: Receiver<Token>,
    buffer: Vec<Token>,
    start_position: usize,
    position: usize,
}

impl Parser {
    fn new(tokens: Receiver<Token>) -> Self {
        return Self {
            tokens,
            buffer: Vec::new(),
            start_position: 0,
            position: 0,
        };
    }

    fn next(&mut self) -> Option<&Token> {
        if let Ok(token) = self.tokens.recv() {
            self.buffer.push(token);
        }

        if self.position >= self.buffer.len() {
            return None;
        }

        let token = &self.buffer[self.position];
        self.position += 1;

        return Some(token);
    }
}

pub fn parse(token_reciever: Receiver<Token>) -> Result<Node, ParserError> {
    let mut parser = Parser::new(token_reciever);

    let children = Vec::new();

    while let Some(token) = parser.next() {
        println!("{:#?}", token);
    }

    return Ok(Node::Root(children));
}
