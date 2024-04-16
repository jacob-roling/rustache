use anyhow::{Error, Result};
use crossbeam_channel::Receiver;
use thiserror::Error;

use crate::{lexer::Token, node::Node};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("unexpected token")]
    UnexpectedToken,
    #[error("unclosed section")]
    UnclosedSection,
}

trait Parser {
    fn next(&mut self) -> Option<&Token>;
}

pub struct ChanneledParser {
    tokens: Receiver<Token>,
    buffer: Vec<Token>,
    position: usize,
}

impl ChanneledParser {
    fn new(tokens: Receiver<Token>) -> Self {
        return Self {
            tokens,
            buffer: Vec::new(),
            position: 0,
        };
    }
}

impl Parser for ChanneledParser {
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

struct NestedParser {
    buffer: Vec<Token>,
    position: usize,
}

impl NestedParser {
    fn new(buffer: Vec<Token>) -> Self {
        return Self {
            buffer,
            position: 0,
        };
    }
}

impl Parser for NestedParser {
    fn next(&mut self) -> Option<&Token> {
        if self.position >= self.buffer.len() {
            return None;
        }

        let token = &self.buffer[self.position];
        self.position += 1;

        return Some(token);
    }
}

pub fn parse(token_reciever: Receiver<Token>) -> Result<Node, ParserError> {
    let parser = ChanneledParser::new(token_reciever);
    return internal_parse(parser);
}

fn internal_parse(mut parser: impl Parser) -> Result<Node, ParserError> {
    let mut error = None;
    let mut children = Vec::new();

    'outer: while let Some(token) = parser.next() {
        match token {
            Token::Error(_) => {}
            Token::Text(value) => children.push(Node::Text(value.into())),
            Token::OpenDelimiter => {
                if let Some(token) = parser.next() {
                    println!("{:#?}", token);
                    match token {
                        Token::Identifier(identifier) => {
                            children.push(Node::Variable {
                                identifier: identifier.into(),
                                escaped: true,
                            });
                        }
                        Token::Section => match parse_section(&mut parser, false) {
                            Ok(node) => {
                                children.push(node);
                            }
                            Err(e) => {
                                error = Some(e);
                                break 'outer;
                            }
                        },
                        Token::Implicit => {}
                        Token::InvertedSection => match parse_section(&mut parser, true) {
                            Ok(node) => {
                                children.push(node);
                            }
                            Err(e) => {
                                error = Some(e);
                                break 'outer;
                            }
                        },
                        Token::Comment(_) => {}
                        Token::Partial => {}
                        Token::Block => {}
                        Token::Parent => {}
                        Token::Raw => {
                            if let Some(token) = parser.next() {
                                match token {
                                    Token::Identifier(identifier) => {
                                        children.push(Node::Variable {
                                            identifier: identifier.into(),
                                            escaped: false,
                                        });
                                    }
                                    _ => {
                                        error = Some(ParserError::UnexpectedToken);
                                        break 'outer;
                                    }
                                }
                            }
                        }
                        _ => {
                            error = Some(ParserError::UnexpectedToken);
                            break 'outer;
                        }
                    }
                }
            }
            Token::CloseDelimiter => {}
            Token::SetDelimiter => {}
            Token::EOF => {}
            _ => {
                return Err(ParserError::UnexpectedToken);
            }
        }
    }

    while let Some(token) = parser.next() {}

    return match error {
        Some(e) => Err(e),
        None => Ok(Node::Root(children)),
    };
}

fn parse_section(parser: &mut impl Parser, inverted: bool) -> Result<Node, ParserError> {
    if let Some(token) = parser.next() {
        match token {
            Token::Identifier(open_identifier) => {
                let mut nested_token_buffer = Vec::new();
                let section_identifier = &open_identifier.clone();

                while let Some(token) = parser.next() {
                    match token {
                        Token::SectionEnd => {
                            if let Some(token) = parser.next() {
                                match token {
                                    Token::Identifier(close_identifier) => {
                                        if section_identifier == close_identifier {
                                            let nested_parser =
                                                NestedParser::new(nested_token_buffer);

                                            match internal_parse(nested_parser) {
                                                Ok(section_root) => {
                                                    if let Node::Root(section_children) =
                                                        section_root
                                                    {
                                                        return Ok(Node::Section {
                                                            identifier: section_identifier.into(),
                                                            inverted,
                                                            children: section_children,
                                                        });
                                                    }
                                                }
                                                Err(error) => return Err(error),
                                            }
                                        } else {
                                            nested_token_buffer.push(token);
                                        }
                                    }
                                    _ => return Err(ParserError::UnexpectedToken),
                                }
                            }
                        }
                        _ => {
                            nested_token_buffer.push(token);
                        }
                    }
                }
            }
            _ => return Err(ParserError::UnexpectedToken),
        }
    }

    return Err(ParserError::UnexpectedToken);
}
