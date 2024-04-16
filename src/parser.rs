use anyhow::Result;
use crossbeam_channel::Receiver;
use thiserror::Error;

use crate::{lexer::Token, node::Node};

pub struct Parser {
    tokens: Receiver<Token>,
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("test")]
    Test,
}

impl Parser {
    pub fn new(tokens: Receiver<Token>) -> Self {
        return Self { tokens };
    }

    pub fn next(&self) -> Token {
        return match self.tokens.recv() {
            Ok(token) => token,
            Err(e) => panic!("{}", e),
        };
    }
}

pub fn parse(token_reciever: Receiver<Token>) -> Result<Node, ParserError> {
    let parser = Parser::new(token_reciever);

    let children = Vec::new();

    while let Ok(token) = parser.tokens.recv() {
        println!("{:#?}", token);
    }

    return Ok(Node::Root(children));
}
