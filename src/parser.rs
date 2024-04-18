use std::collections::VecDeque;

use anyhow::Result;
use crossbeam_channel::Receiver;
use thiserror::Error;

use crate::{lexer::Token, node::Node};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("{line}:{column} syntax error: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },
    #[error("unexpected token: {0:#?}")]
    UnexpectedToken(Token),
    #[error("expected token: {0:#?} got nothing")]
    ExpectedToken(Token),
    #[error("expected token: {0:#?} got {1:#?}")]
    ExpectedTokenGot(Token, Token),
}

struct Parser {
    tokens: Option<Receiver<Token>>,
    buffer: VecDeque<Token>,
}

impl Parser {
    fn new(tokens: Option<Receiver<Token>>) -> Self {
        return Self {
            tokens,
            buffer: VecDeque::new(),
        };
    }

    fn next(&mut self) -> Option<Token> {
        if let Some(token_receiver) = &self.tokens {
            if let Ok(token) = token_receiver.recv() {
                self.buffer.push_back(token);
            }
        }

        return self.buffer.pop_front();
    }

    fn parse(&mut self) -> Result<Vec<Node>, ParserError> {
        let mut nodes = Vec::new();

        while let Some(token) = self.next() {
            match token {
                Token::Error {
                    line,
                    column,
                    message,
                } => {
                    return Err(ParserError::SyntaxError {
                        line,
                        column,
                        message,
                    })
                }
                Token::Text(text) => nodes.push(Node::Text(text)),
                Token::OpenDelimiter => {
                    if let Some(token) = self.next() {
                        match token {
                            Token::Block
                            | Token::Section
                            | Token::InvertedSection
                            | Token::Parent => match self.parse_section() {
                                Ok(children) => {}
                                Err(error) => return Err(error),
                            },
                            Token::Identifier(identifier) => {
                                if let Some(token) = self.next() {
                                    if let Token::CloseDelimiter = token {
                                        nodes.push(Node::Variable {
                                            identifier,
                                            escaped: true,
                                        })
                                    } else {
                                        return Err(ParserError::ExpectedTokenGot(
                                            Token::CloseDelimiter,
                                            token,
                                        ));
                                    }
                                } else {
                                    return Err(ParserError::ExpectedToken(Token::CloseDelimiter));
                                }
                            }
                            Token::Raw => {
                                if let Some(token) = self.next() {
                                    if let Token::Identifier(identifier) = token {
                                        if let Some(token) = self.next() {
                                            if let Token::CloseDelimiter = token {
                                                nodes.push(Node::Variable {
                                                    identifier,
                                                    escaped: false,
                                                })
                                            } else {
                                                return Err(ParserError::ExpectedTokenGot(
                                                    Token::CloseDelimiter,
                                                    token,
                                                ));
                                            }
                                        } else {
                                            return Err(ParserError::ExpectedToken(
                                                Token::CloseDelimiter,
                                            ));
                                        }
                                    } else {
                                        return Err(ParserError::UnexpectedToken(token));
                                    }
                                }
                            }
                            Token::Implicit => nodes.push(Node::Implicit),
                            _ => return Err(ParserError::UnexpectedToken(token)),
                        }
                    }
                }
                Token::CloseDelimiter => {}
                Token::EOF => {}
                _ => return Err(ParserError::UnexpectedToken(token)),
            }
        }

        return Ok(nodes);
    }

    fn parse_section(&mut self) -> Result<Vec<Node>, ParserError> {
        return Ok(Vec::new());
    }
}

pub fn parse(token_reciever: Receiver<Token>) -> Result<Vec<Node>, ParserError> {
    let mut parser = Parser::new(Some(token_reciever));
    return parser.parse();
}
