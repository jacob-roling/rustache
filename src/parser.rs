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
    #[error("unclosed section: {0}")]
    UnclosedSection(String),
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
                Token::CloseDelimiter => {}
                Token::EOF => {}
                Token::Text(text) => nodes.push(Node::Text(text)),
                Token::OpenDelimiter => {
                    if let Some(token) = self.next() {
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
                            Token::Comment(comment) => nodes.push(Node::Comment(comment)),
                            Token::SetDelimiter => {}
                            Token::Partial => {
                                if let Some(token) = self.next() {
                                    match token {
                                        Token::Identifier(identifier) => {
                                            nodes.push(Node::Partial {
                                                identifier,
                                                dynamic: false,
                                            });
                                        }
                                        Token::Dynamic => {
                                            if let Some(Token::Identifier(identifier)) = self.next()
                                            {
                                                nodes.push(Node::Partial {
                                                    identifier,
                                                    dynamic: true,
                                                });
                                            }
                                        }
                                        _ => return Err(ParserError::UnexpectedToken(token)),
                                    }
                                }
                            }
                            Token::Section => {
                                if let Some(Token::Identifier(identifier)) = self.next() {
                                    if let Some(tokens) = self.section_tokens(&identifier) {
                                        let mut sub_parser = Parser::new(None);
                                        sub_parser.buffer = tokens.into();

                                        match sub_parser.parse() {
                                            Ok(children) => {
                                                nodes.push(Node::Section {
                                                    identifier,
                                                    inverted: false,
                                                    children,
                                                });
                                            }
                                            Err(error) => return Err(error),
                                        }
                                    } else {
                                        return Err(ParserError::UnclosedSection(identifier));
                                    }
                                }
                            }
                            Token::InvertedSection => {
                                if let Some(Token::Identifier(identifier)) = self.next() {
                                    if let Some(tokens) = self.section_tokens(&identifier) {
                                        let mut sub_parser = Parser::new(None);
                                        sub_parser.buffer = tokens.into();

                                        match sub_parser.parse() {
                                            Ok(children) => {
                                                nodes.push(Node::Section {
                                                    identifier,
                                                    inverted: true,
                                                    children,
                                                });
                                            }
                                            Err(error) => return Err(error),
                                        }
                                    } else {
                                        return Err(ParserError::UnclosedSection(identifier));
                                    }
                                }
                            }
                            Token::Block => {
                                if let Some(Token::Identifier(identifier)) = self.next() {
                                    if let Some(tokens) = self.section_tokens(&identifier) {
                                        let mut sub_parser = Parser::new(None);
                                        sub_parser.buffer = tokens.into();

                                        match sub_parser.parse() {
                                            Ok(children) => {
                                                nodes.push(Node::Block {
                                                    identifier,
                                                    children,
                                                });
                                            }
                                            Err(error) => return Err(error),
                                        }
                                    } else {
                                        return Err(ParserError::UnclosedSection(identifier));
                                    }
                                }
                            }
                            Token::Parent => {
                                if let Some(token) = self.next() {
                                    match token {
                                        Token::Identifier(identifier) => {
                                            if let Some(tokens) = self.section_tokens(&identifier) {
                                                let mut sub_parser = Parser::new(None);
                                                sub_parser.buffer = tokens.into();

                                                match sub_parser.parse() {
                                                    Ok(children) => {
                                                        nodes.push(Node::Parent {
                                                            identifier,
                                                            dynamic: false,
                                                            children,
                                                        });
                                                    }
                                                    Err(error) => return Err(error),
                                                }
                                            } else {
                                                return Err(ParserError::UnclosedSection(
                                                    identifier,
                                                ));
                                            }
                                        }
                                        Token::Dynamic => {
                                            if let Some(Token::Identifier(identifier)) = self.next()
                                            {
                                                if let Some(tokens) =
                                                    self.section_tokens(&identifier)
                                                {
                                                    let mut sub_parser = Parser::new(None);
                                                    sub_parser.buffer = tokens.into();

                                                    match sub_parser.parse() {
                                                        Ok(children) => {
                                                            nodes.push(Node::Parent {
                                                                identifier,
                                                                dynamic: true,
                                                                children,
                                                            });
                                                        }
                                                        Err(error) => return Err(error),
                                                    }
                                                } else {
                                                    return Err(ParserError::UnclosedSection(
                                                        identifier,
                                                    ));
                                                }
                                            }
                                        }
                                        _ => return Err(ParserError::UnexpectedToken(token)),
                                    }
                                }
                            }
                            Token::Identifier(identifier) => {
                                nodes.push(Node::Variable {
                                    identifier,
                                    escaped: true,
                                });
                            }
                            Token::Raw => {
                                if let Some(Token::Identifier(identifier)) = self.next() {
                                    nodes.push(Node::Variable {
                                        identifier,
                                        escaped: false,
                                    })
                                }
                            }
                            Token::Implicit => nodes.push(Node::Implicit),
                            _ => return Err(ParserError::UnexpectedToken(token)),
                        }
                    }
                }
                _ => return Err(ParserError::UnexpectedToken(token)),
            }
        }

        return Ok(nodes);
    }

    fn section_tokens(&mut self, identifier: &String) -> Option<Vec<Token>> {
        let mut tokens = Vec::new();

        if let Some(Token::CloseDelimiter) = self.next() {
        } else {
            return None;
        }

        let mut next_token = None;
        let mut next_next_token = None;

        while let Some(token) = self.next() {
            if let Token::SectionEnd = token {
                if let Some(token) = self.next() {
                    if let Token::Identifier(ref other_identifier) = token {
                        if identifier == other_identifier {
                            // Pop off the last open delimiter
                            tokens.pop();
                            return Some(tokens);
                        }
                    }
                    if let Token::Dynamic = token {
                        if let Some(token) = self.next() {
                            if let Token::Identifier(ref other_identifier) = token {
                                if identifier == other_identifier {
                                    // Pop off the last open delimiter
                                    tokens.pop();
                                    return Some(tokens);
                                }
                            }
                            next_next_token = Some(token);
                        }
                    }
                    next_token = Some(token);
                }
            }
            tokens.push(token);
            if next_token.is_some() {
                tokens.push(next_token.take().unwrap());
            }
            if next_next_token.is_some() {
                tokens.push(next_next_token.take().unwrap());
            }
        }

        return None;
    }
}

pub fn parse(token_reciever: Receiver<Token>) -> Result<Vec<Node>, ParserError> {
    let mut parser = Parser::new(Some(token_reciever));
    return parser.parse();
}
