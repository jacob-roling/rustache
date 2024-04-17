use std::collections::VecDeque;

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
    #[error("expected token")]
    ExpectedToken,
}

trait State {
    fn next(&mut self, parser: &mut Parser) -> StateFunction;
}

type StateFunction = Option<Box<dyn State>>;

pub struct Parser {
    tokens: Option<Receiver<Token>>,
    buffer: Vec<Token>,
    position: usize,
    start_position: usize,
    result: Vec<Node>,
}

pub fn parse(tokens: Receiver<Token>) {
    let mut parser = Parser::new();
    parser.tokens = Some(tokens);
    internal_parse(parser);
}

fn internal_parse(mut parser: Parser) {
    let mut state_function: StateFunction = Some(Box::new(ParseText));
    while let Some(mut state) = state_function {
        state_function = state.next(&mut parser);
    }
}

impl Parser {
    fn new() -> Self {
        return Self {
            tokens: None,
            buffer: Vec::new(),
            position: 0,
            start_position: 0,
            result: Vec::new(),
        };
    }

    fn next(&mut self) -> Option<Token> {
        if let Some(receiver) = &self.tokens {
            if let Ok(token) = receiver.recv() {
                self.buffer.push(token);
            }
        }

        if self.position >= self.buffer.len() {
            return None;
        }

        let token = self.buffer[self.position].clone();
        self.position += 1;

        return Some(token);
    }

    fn backup(&mut self, count: usize) {
        self.position -= count;
    }

    fn peek(&mut self) -> Option<Token> {
        return match self.next() {
            Some(token) => {
                self.backup(1);
                return Some(token);
            }
            None => None,
        };
    }

    fn emit(&mut self, node: Node) {
        self.result.push(node);
    }
}

struct ParseText;

impl State for ParseText {
    fn next(&mut self, parser: &mut Parser) -> StateFunction {
        while let Some(token) = parser.next() {
            match token {
                Token::Text(text) => {
                    parser.emit(Node::Text(text));
                }
                Token::OpenDelimiter => return Some(Box::new(ParseDelimiter)),
                Token::Error(error) => {}
                Token::EOF => {}
                _ => return None,
            }
        }
        return None;
    }
}

struct ParseDelimiter;

impl State for ParseDelimiter {
    fn next(&mut self, parser: &mut Parser) -> StateFunction {
        if let Some(token) = parser.next() {
            match token {
                Token::Comment(comment) => parser.emit(Node::Comment(comment)),
                _ => {}
            }
        }
        return None;
    }
}

// impl ChanneledParser {
//     fn new(tokens: Receiver<Token>) -> Self {
//         return Self {
//             tokens,
//             buffer: Vec::new(),
//             position: 0,
//         };
//     }
// }

// impl Parser for ChanneledParser {
//     fn next(&mut self) -> Option<&Token> {
// if let Ok(token) = self.tokens.recv() {
//     self.buffer.push(token);
// }

// if self.position >= self.buffer.len() {
//     return None;
// }

// let token = &self.buffer[self.position];
// self.position += 1;

// return Some(token);
//     }
// }

// struct NestedParser {
//     buffer: Vec<Token>,
//     position: usize,
// }

// impl NestedParser {
//     fn new(buffer: Vec<Token>) -> Self {
//         return Self {
//             buffer,
//             position: 0,
//         };
//     }
// }

// impl Parser for NestedParser {
//     fn next(&mut self) -> Option<&Token> {
//         if self.position >= self.buffer.len() {
//             return None;
//         }

//         let token = &self.buffer[self.position];
//         self.position += 1;

//         return Some(token);
//     }
// }

// pub fn parse(token_reciever: Receiver<Token>) -> Result<Node, ParserError> {
//     let parser = ChanneledParser::new(token_reciever);
//     return internal_parse(parser);
// }

// fn internal_parse(mut parser: impl Parser) -> Result<Node, ParserError> {
//     let mut error = None;
//     let mut children = Vec::new();

//     'outer: while let Some(token) = parser.next() {
//         match token {
//             Token::Error(_) => {}
//             Token::Text(value) => children.push(Node::Text(value.into())),
//             Token::OpenDelimiter => {
//                 if let Some(token) = parser.next() {
//                     println!("{:#?}", token);
//                     match token {
//                         Token::Identifier(identifier) => {
//                             children.push(Node::Variable {
//                                 identifier: identifier.into(),
//                                 escaped: true,
//                             });
//                         }
//                         Token::Section => match parse_section(&mut parser, false) {
//                             Ok(node) => {
//                                 children.push(node);
//                             }
//                             Err(e) => {
//                                 error = Some(e);
//                                 break 'outer;
//                             }
//                         },
//                         Token::Implicit => {}
//                         Token::InvertedSection => match parse_section(&mut parser, true) {
//                             Ok(node) => {
//                                 children.push(node);
//                             }
//                             Err(e) => {
//                                 error = Some(e);
//                                 break 'outer;
//                             }
//                         },
//                         Token::Comment(_) => {}
//                         Token::Partial => {}
//                         Token::Block => {}
//                         Token::Parent => {}
//                         Token::Raw => {
//                             if let Some(token) = parser.next() {
//                                 match token {
//                                     Token::Identifier(identifier) => {
//                                         children.push(Node::Variable {
//                                             identifier: identifier.into(),
//                                             escaped: false,
//                                         });
//                                     }
//                                     _ => {
//                                         error = Some(ParserError::UnexpectedToken);
//                                         break 'outer;
//                                     }
//                                 }
//                             }
//                         }
//                         _ => {
//                             error = Some(ParserError::UnexpectedToken);
//                             break 'outer;
//                         }
//                     }
//                 }
//             }
//             Token::CloseDelimiter => {}
//             Token::SetDelimiter => {}
//             Token::EOF => {}
//             _ => {
//                 return Err(ParserError::UnexpectedToken);
//             }
//         }
//     }

//     while let Some(_) = parser.next() {}

//     return match error {
//         Some(e) => Err(e),
//         None => Ok(Node::Root(children)),
//     };
// }

// fn parse_section(parser: &mut impl Parser, inverted: bool) -> Result<Node, ParserError> {
//     let maybe_section_identifier = parser.next().and_then(|t| {
//         if let Token::Identifier(identifier) = t {
//             return Some(identifier);
//         }
//         return None;
//     });

//     if let Some(section_name) = maybe_section_identifier {
//         let mut nested_token_buffer: Vec<Token> = Vec::new();

//         while let Some(token) = parser.next() {
//             match token {
//                 Token::SectionEnd => {
//                     let maybe_end_identifier = parser.next().and_then(|t| {
//                         if let Token::Identifier(identifier) = t {
//                             return Some(identifier);
//                         }
//                         return None;
//                     });

//                     // if maybe_end_identifier.is_some_and(|section_end_identifier| {
//                     //     section_name == section_end_identifier
//                     // }) {
//                     //     break;
//                     // }
//                 }
//                 _ => nested_token_buffer.push(token.clone()),
//             }
//         }

//         println!("{:#?}", nested_token_buffer);
//     }

//     return Err(ParserError::UnexpectedToken);

//                 let identifier = parser.next().and_then(|t| {
//                     if let Token::Identifier(identifier) = t {
//                         return Some(identifier);
//                     }
//                     return None;
//                 });

//                 if let Some(section_end_name) = identifier {
//                     if section_name == section_end_name {
//                         break;
//                     }
//                 }
//             }
//     if let Some(token) = parser.next() {
//         match token {
//             Token::Identifier(open_identifier) => {
//                 while let Some(token) = parser.next() {
//                     match token {
//                         Token::SectionEnd => {
//                             if let Some(token) = parser.next() {
//                                 match token {
//                                     Token::Identifier(close_identifier) => {
//                                         if section_identifier == close_identifier {
//                                             let nested_parser =
//                                                 NestedParser::new(nested_token_buffer);

//                                             match internal_parse(nested_parser) {
//                                                 Ok(section_root) => {
//                                                     if let Node::Root(section_children) =
//                                                         section_root
//                                                     {
//                                                         return Ok(Node::Section {
//                                                             identifier: section_identifier.into(),
//                                                             inverted,
//                                                             children: section_children,
//                                                         });
//                                                     }
//                                                 }
//                                                 Err(error) => return Err(error),
//                                             }
//                                         } else {
//                                             nested_token_buffer.push(token.clone());
//                                         }
//                                     }
//                                     _ => return Err(ParserError::UnexpectedToken),
//                                 }
//                             }
//                         }
//                         _ => {
//                             nested_token_buffer.push(token.clone());
//                         }
//                     }
//                 }
//             }
//             _ => return Err(ParserError::UnexpectedToken),
//         }
//     }
// }
