use crossbeam_channel::Sender;
use std::io::{self, BufRead, BufReader};
use substring::Substring;
use thiserror::Error;

const VALID_IDENTIFIER_CHARACTER_SET: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.?_";

const VALID_DELIMITER_CHARACTER_SET: &str = "<>%()@";

#[derive(Debug, Clone)]
pub enum Token {
    Error(LexerError),
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
    SectionEnd,
}

#[derive(Error, Debug, Clone)]
pub enum LexerError {
    #[error("unclosed delimiter")]
    UnclosedDelimiter,
    #[error("unexpected end of file")]
    UnexpectedEOF,
    #[error("unexpected character: {0}")]
    UnexpectedCharacter(char),
    #[error("TODO")]
    TODO,
}

pub struct Lexer<R> {
    reader: BufReader<R>,
    buffer: String,
    tokens: Sender<Token>,
    start_position: usize,
    position: usize,
    open_delimiter: String,
    open_delimiter_chars: usize,
    close_delimiter: String,
    close_delimiter_chars: usize,
    raw_close_delimiter: String,
    raw_close_delimiter_chars: usize,
}

trait State<R> {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R>;
}

type StateFunction<R> = Option<Box<dyn State<R>>>;

pub fn lex<R: io::Read>(reader: BufReader<R>, sender: Sender<Token>) {
    let mut lexer = Lexer::new(reader, sender);
    let mut state_function: StateFunction<R> = Some(Box::new(LexText));
    while let Some(mut state) = state_function {
        state_function = state.next(&mut lexer);
    }
}

struct LexText;

impl<R: io::Read> State<R> for LexText {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        loop {
            if lexer.peekn(lexer.open_delimiter_chars) == lexer.open_delimiter {
                if lexer.position > lexer.start_position {
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
        lexer.nextn(lexer.open_delimiter_chars);
        lexer.emit(Token::OpenDelimiter);
        return Some(Box::new(LexInsideDelimiter));
    }
}

struct LexInsideDelimiter;

impl<R: io::Read> State<R> for LexInsideDelimiter {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        return match lexer.next() {
            Some(next_character) => match next_character {
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
                    lexer.emit(Token::Raw);
                    return Some(Box::new(LexRawIdentifier));
                }
                '&' => {
                    lexer.emit(Token::Raw);
                    return Some(Box::new(LexIdentifier));
                }
                '>' => {
                    lexer.emit(Token::Partial);
                    return Some(Box::new(LexIdentifier));
                }
                '=' => {
                    lexer.emit(Token::SetDelimiter);
                    return Some(Box::new(LexNewDelimiter));
                }
                '\n' => return lexer.emit_error(LexerError::UnclosedDelimiter),
                next_character if next_character.is_alphanumeric() => {
                    lexer.backup(1);
                    return Some(Box::new(LexIdentifier));
                }
                _ => lexer.emit_error(LexerError::UnexpectedCharacter(next_character)),
            },
            None => lexer.emit_error(LexerError::UnexpectedEOF),
        };
    }
}

struct LexCloseDelimiter;

impl<R: io::Read> State<R> for LexCloseDelimiter {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        // Ignore whitespace
        lexer.accept_run(" ");
        lexer.ignore();

        // Peek to see if we have reached a closing delimiter
        if lexer.peekn(lexer.close_delimiter_chars) == lexer.close_delimiter {
            lexer.ignore();
            lexer.nextn(lexer.close_delimiter_chars);
            lexer.emit(Token::CloseDelimiter);
            return Some(Box::new(LexText));
        }

        // If not, we either hit an unexpected character or EOF
        return match lexer.next() {
            None => lexer.emit_error(LexerError::UnexpectedEOF),
            Some(character) => lexer.emit_error(LexerError::UnexpectedCharacter(character)),
        };
    }
}

struct LexIdentifier;

impl<R: io::Read> State<R> for LexIdentifier {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        // Ignore whitespace
        lexer.accept_run(" ");
        lexer.ignore();

        // Check if identifier is dynamic
        if lexer.accept("*") {
            lexer.emit(Token::Dynamic);
        }

        // Consume valid characters and emit
        lexer.accept_run(VALID_IDENTIFIER_CHARACTER_SET);
        lexer.emit(Token::Identifier(lexer.current()));

        return Some(Box::new(LexCloseDelimiter));
    }
}

struct LexRawIdentifier;

impl<R: io::Read> State<R> for LexRawIdentifier {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        // Ignore whitespace
        lexer.accept_run(" ");
        lexer.ignore();

        // Consume valid characters and emit
        lexer.accept_run(VALID_IDENTIFIER_CHARACTER_SET);
        lexer.emit(Token::Identifier(lexer.current()));

        return Some(Box::new(LexCloseRawDelimiter));
    }
}

struct LexComment;

impl<R: io::Read> State<R> for LexComment {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        lexer.ignore();
        loop {
            if lexer.peekn(lexer.close_delimiter_chars) == lexer.close_delimiter {
                lexer.emit(Token::Comment(lexer.current()));
                lexer.nextn(lexer.close_delimiter_chars);
                lexer.emit(Token::CloseDelimiter);
                return Some(Box::new(LexText));
            }
            if lexer.next().is_none() {
                break;
            }
        }
        return lexer.emit_error(LexerError::UnexpectedEOF);
    }
}

struct LexCloseRawDelimiter;

impl<R: io::Read> State<R> for LexCloseRawDelimiter {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        // Ignore whitespace
        lexer.accept_run(" ");
        lexer.ignore();

        // Peek to see if we have reached a raw closing delimiter
        if lexer.peekn(lexer.raw_close_delimiter_chars) == lexer.raw_close_delimiter {
            lexer.nextn(lexer.raw_close_delimiter_chars);
            lexer.emit(Token::CloseDelimiter);
            return Some(Box::new(LexText));
        }

        // If not, we either hit an unexpected character or EOF
        return match lexer.next() {
            None => lexer.emit_error(LexerError::UnexpectedEOF),
            Some(character) => lexer.emit_error(LexerError::UnexpectedCharacter(character)),
        };
    }
}

struct LexNewDelimiter;

impl<R: io::Read> State<R> for LexNewDelimiter {
    fn next(&mut self, lexer: &mut Lexer<R>) -> StateFunction<R> {
        // Ignore whitespace
        lexer.accept_run(" ");
        lexer.ignore();

        // Consume valid characters and set new open delimiter
        lexer.accept_run(VALID_DELIMITER_CHARACTER_SET);
        let new_open_delimiter = lexer.current();
        lexer.ignore();

        if !lexer.accept(" ") {
            return lexer.emit_error(LexerError::TODO);
        }
        lexer.ignore();

        // Consume valid characters and set new close delimiter
        lexer.accept_run(VALID_DELIMITER_CHARACTER_SET);
        let new_close_delimiter = lexer.current();
        lexer.ignore();

        if !lexer.accept("=") {
            return lexer.emit_error(LexerError::TODO);
        }

        // Peek to see if we have reached the old closing delimiter
        if lexer.peekn(lexer.close_delimiter_chars) == lexer.close_delimiter {
            lexer.ignore();
            lexer.nextn(lexer.close_delimiter_chars);
            lexer.emit(Token::CloseDelimiter);
            // Set new delimiters
            lexer.set_delimiters(new_open_delimiter, new_close_delimiter);
            return Some(Box::new(LexText));
        }

        // If not, we either hit an unexpected character or EOF
        return match lexer.next() {
            None => lexer.emit_error(LexerError::UnexpectedEOF),
            Some(character) => lexer.emit_error(LexerError::UnexpectedCharacter(character)),
        };
    }
}

impl<R: io::Read> Lexer<R> {
    fn new(reader: BufReader<R>, sender: Sender<Token>) -> Self {
        let open_delimiter = String::from("{{");
        let close_delimiter = String::from("}}");

        return Self {
            reader,
            buffer: String::new(),
            tokens: sender,
            start_position: 0,
            position: 0,
            open_delimiter: open_delimiter.clone(),
            open_delimiter_chars: open_delimiter.chars().count(),
            close_delimiter: close_delimiter.clone(),
            close_delimiter_chars: close_delimiter.chars().count(),
            raw_close_delimiter: String::from("}}}"),
            raw_close_delimiter_chars: 3,
        };
    }

    fn next(&mut self) -> Option<char> {
        match self.reader.fill_buf() {
            Ok(buffer) => {
                let length = buffer.len();
                if length > 0 {
                    match std::str::from_utf8(buffer) {
                        Ok(next_string) => {
                            // println!("READ IN: {}", next_string);
                            self.buffer.push_str(next_string);
                        }
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    }
                }
                self.reader.consume(length);
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

    fn nextn(&mut self, count: usize) -> String {
        let mut result = String::new();
        for _ in 0..count {
            let Some(next_character) = self.next() else {
                break;
            };
            result.push(next_character);
        }
        return result;
    }

    fn ignore(&mut self) {
        self.start_position = self.position;
    }

    fn backup(&mut self, characters: usize) {
        self.position -= characters;
    }

    fn peekn(&mut self, count: usize) -> String {
        let mut result = String::new();
        let mut num_added = 0;
        for _ in 0..count {
            if let Some(next_characteracter) = self.next() {
                result.push(next_characteracter);
                num_added += 1;
            }
        }
        self.backup(num_added);
        return result;
    }

    fn emit(&mut self, token: Token) {
        self.start_position = self.position;

        if let Err(error) = self.tokens.send(token) {
            panic!("{}", error);
        }
    }

    fn current(&self) -> String {
        return self
            .buffer
            .substring(self.start_position, self.position)
            .to_string();
    }

    fn emit_error(&mut self, error: LexerError) -> StateFunction<R> {
        self.tokens.send(Token::Error(error.into())).unwrap();
        return None;
    }

    fn accept(&mut self, character_set: &str) -> bool {
        if let Some(next_characteracter) = self.next() {
            if character_set.contains(next_characteracter) {
                return true;
            }
        }
        self.backup(1);
        return false;
    }

    fn accept_run(&mut self, character_set: &str) {
        while let Some(next_characteracter) = self.next() {
            if !character_set.contains(next_characteracter) {
                break;
            }
        }
        self.backup(1);
    }

    fn set_delimiters(&mut self, open_delimiter: String, close_delimiter: String) {
        self.open_delimiter = open_delimiter;
        self.open_delimiter_chars = self.open_delimiter.chars().count();
        self.close_delimiter = close_delimiter;
        self.close_delimiter_chars = self.close_delimiter.chars().count();
        let mut raw_close_delimiter = self.close_delimiter.clone();
        raw_close_delimiter.insert(0, '}');
        self.raw_close_delimiter = raw_close_delimiter;
        self.raw_close_delimiter_chars = self.raw_close_delimiter.chars().count();
    }
}
