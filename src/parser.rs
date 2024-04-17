use std::collections::VecDeque;

use crate::lexer::Token;

struct Parser {
    buffer: VecDeque<Token>,
}
