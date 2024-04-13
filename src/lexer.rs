use anyhow::Error;
use std::{
    io::{self, BufRead, BufReader},
    sync::mpsc::SyncSender,
};
use substring::Substring;
use thiserror::Error;

#[derive(Debug)]
pub enum Token {
    Error(Error),
    EOF,
    Text(String),
    OpenDelimiter,
    CloseDelimiter,
    Section,
    InvertedSection,
    Identifier(String),
    Implicit,
    Comment(String),
    Partial,
    Dynamic,
    Block,
    Parent,
    SetDelimiter,
    Raw,
    OpenRawDelimiter,
    CloseRawDelimiter,
    SectionEnd,
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("unclosed delimiter")]
    UnclosedDelimiter,
    #[error("unexpected end of file")]
    UnexpectedEOF,
    #[error("unexpected character")]
    UnexpectedCharacter,
}

pub struct Lexer<R> {
    input: BufReader<R>,
    buffer: String,
    tokens: SyncSender<Token>,
    start_position: usize,
    position: usize,
    open_delimiter: String,
    close_delimiter: String,
}

pub trait State<R> {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R>;
}

type StateFunction<R> = Option<Box<dyn State<R>>>;

pub fn lex<R: io::Read>(input: BufReader<R>, sender: SyncSender<Token>) {
    let mut lexer = Lexer::new(input, sender);
    let mut state_function: StateFunction<R> = Some(Box::new(LexText));
    while let Some(mut state) = state_function {
        state_function = state.next(&mut lexer);
    }
}

struct LexText;

impl<R: io::Read> State<R> for LexText {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        for _ in lexer.current().len()..lexer.open_delimiter.len() {
            lexer.next();
        }
        loop {
            if lexer.current().ends_with(lexer.open_delimiter.as_str()) {
                if lexer.position > lexer.start_position {
                    lexer.backup(lexer.open_delimiter.len());
                    lexer.emit(Token::Text(lexer.current()));
                }
                return Some(Box::new(LexOpenDelimiter));
            }
            if lexer.next().is_none() {
                break;
            }
        }
        if lexer.position > lexer.start_position {
            lexer.emit(Token::Text(lexer.current()));
        }
        lexer.emit(Token::EOF);
        return None;
    }
}

struct LexOpenDelimiter;

impl<R: io::Read> State<R> for LexOpenDelimiter {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        lexer.next();
        lexer.next();
        lexer.emit(Token::OpenDelimiter);
        return Some(Box::new(LexInsideDelimiter));
    }
}

struct LexInsideDelimiter;

impl<R: io::Read> State<R> for LexInsideDelimiter {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        for _ in lexer.current().len()..lexer.close_delimiter.len() {
            lexer.next();
        }
        loop {
            if lexer.current().ends_with(lexer.close_delimiter.as_str()) {
                println!("ASSAD {}", lexer.current());
                lexer.backup(lexer.close_delimiter.len());
                return Some(Box::new(LexCloseDelimiter));
            }
            match lexer.next() {
                Some(next_char) => match next_char {
                    '#' => {
                        lexer.emit(Token::Section);
                        return Some(Box::new(LexIdentifier));
                    }
                    '.' => {
                        lexer.emit(Token::Implicit);
                        return Some(Box::new(LexCloseDelimiter));
                    }
                    '!' => {
                        return Some(Box::new(LexComment));
                    }
                    '/' => {
                        lexer.emit(Token::SectionEnd);
                        return Some(Box::new(LexIdentifier));
                    }
                    '$' => {
                        lexer.emit(Token::Block);
                        return Some(Box::new(LexIdentifier));
                    }
                    '^' => {
                        lexer.emit(Token::InvertedSection);
                        return Some(Box::new(LexIdentifier));
                    }
                    '<' => {
                        lexer.emit(Token::Parent);
                        return Some(Box::new(LexIdentifier));
                    }
                    '{' => {
                        lexer.emit(Token::OpenRawDelimiter);
                        return Some(Box::new(LexIdentifier));
                    }
                    '&' => {
                        lexer.emit(Token::Raw);
                        return Some(Box::new(LexIdentifier));
                    }
                    next_char if next_char == 0xA as char => {
                        return lexer.emit_error(LexerError::UnexpectedCharacter)
                    }
                    next_char if next_char.is_whitespace() => lexer.ignore(),
                    _ => return lexer.emit_error(LexerError::UnexpectedCharacter),
                },
                None => return lexer.emit_error(LexerError::UnexpectedEOF),
            };
        }
    }
}

struct LexCloseDelimiter;

impl<R: io::Read> State<R> for LexCloseDelimiter {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        println!("{}", lexer.buffer);
        lexer.emit(Token::CloseDelimiter);
        return Some(Box::new(LexText));
    }
}

struct LexIdentifier;

impl<R: io::Read> State<R> for LexIdentifier {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        return None;
    }
}

struct LexComment;

impl<R: io::Read> State<R> for LexComment {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        return None;
    }
}

struct LexCloseRawDelimiter;

impl<R: io::Read> State<R> for LexCloseRawDelimiter {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        return None;
    }
}

impl<R: io::Read> Lexer<R> {
    pub fn new(input: BufReader<R>, sender: SyncSender<Token>) -> Self {
        return Self {
            input,
            buffer: String::new(),
            tokens: sender,
            start_position: 0,
            position: 0,
            open_delimiter: "{{".to_string(),
            close_delimiter: "}}".to_string(),
        };
    }

    pub fn next(&mut self) -> Option<char> {
        match self.input.fill_buf() {
            Ok(buffer) => {
                let length = buffer.len();
                if length > 0 {
                    match std::str::from_utf8(buffer) {
                        Ok(next_string) => {
                            self.buffer.push_str(next_string);
                        }
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    }
                }
                self.input.consume(length);
            }
            Err(e) => panic!("Reading file: {}", e),
        };

        if self.position >= self.buffer.len() {
            return None;
        }

        let character = self.buffer.chars().nth(self.position).unwrap();
        self.position += 1;

        return Some(character);
    }

    pub fn ignore(&mut self) {
        self.start_position = self.position;
    }

    pub fn backup(&mut self, characters: usize) {
        self.position -= characters;
    }

    pub fn peek(&mut self) -> Option<char> {
        let next_character = self.next();
        self.backup(1);
        return next_character;
    }

    pub fn emit(&mut self, token: Token) {
        self.start_position = self.position;
        self.tokens.send(token).unwrap();
    }

    pub fn current(&self) -> String {
        return self
            .buffer
            .substring(self.start_position, self.position)
            .to_string();
    }

    pub fn emit_error(
        &mut self,
        error: impl std::error::Error + Send + Sync + 'static,
    ) -> StateFunction<R> {
        self.tokens.send(Token::Error(Error::new(error))).unwrap();
        return None;
    }
}
