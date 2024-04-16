use anyhow::{Error, Result};
use crossbeam_channel::Receiver;
use thiserror::Error;

use crate::{lexer::Token, node::Node};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("unexpected token")]
    UnexpectedToken,
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

    // fn peek(&mut self) -> Option<&Token> {
    //     let maybe_token = self.next();
    //     self.backup(1);
    //     return maybe_token;
    // }

    // fn backup(&mut self, count: usize) {
    //     self.position -= count;
    // }
}

pub fn parse(token_reciever: Receiver<Token>) -> Result<Node, ParserError> {
    let mut parser = Parser::new(token_reciever);

    let mut children = Vec::new();

    while let Some(token) = parser.next() {
        match token {
            Token::Error(_) => {}
            Token::Text(value) => children.push(Node::Text(value.clone())),
            Token::OpenDelimiter => {
                if let Some(token) = parser.next() {
                    match token {
                        Token::Identifier(_) => {
                            let node = parse_variable(&mut parser);
                        }
                        Token::Section => {}
                        Token::Implicit => {}
                        Token::InvertedSection => {}
                        Token::Comment(_) => {}
                        Token::Partial => {}
                        Token::Block => {}
                        Token::Parent => {}
                        Token::SetDelimiter => {}
                        Token::Raw => {}
                        _ => {
                            return Err(ParserError::UnexpectedToken);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    return Ok(Node::Root(children));
}

fn parse_variable(parser: &mut Parser) -> Result<Node, ParserError> {
    return Ok(Node::Implicit);
}
