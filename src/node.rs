use std::{
    collections::HashMap,
    io::{BufWriter, Write},
};

use thiserror::Error;

pub type Data = HashMap<String, Value>;

pub enum Value {
    String(String),
    Number(f32),
    Array(Vec<Value>),
    Bool(bool),
    Object(Data),
    Lambda(fn(current_context: Option<Data>) -> String),
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("identifier does not exist in context")]
    IdentifierDoesNotExist,
}

#[derive(Debug)]
pub enum Node {
    Root(Vec<Node>),
    Section {
        identifier: String,
        inverted: bool,
        children: Vec<Node>,
    },
    Variable {
        identifier: String,
        escaped: bool,
    },
    Text(String),
    Implicit,
    Comment(String),
    Partial {
        identifier: String,
        dynamic: bool,
    },
    Block {
        identifier: String,
        children: Vec<Node>,
    },
    Parent {
        identifier: String,
        dynamic: bool,
        children: Vec<Node>,
    },
}

impl Node {
    pub fn render(
        &self,
        writer: &mut BufWriter<impl std::io::Write>,
        data: &Data,
    ) -> Option<RenderError> {
        match self {
            Node::Root(children) => {
                for i in 0..children.len() {
                    children[i].render(writer, data);
                }
            }
            Node::Text(text) => {
                writer.write(text.as_bytes()).unwrap();
            }
            _ Node::Variable{ identifier, escaped } => {
                writer.write(text.as_bytes()).unwrap();
            }
            _ => {}
        }
        return None;
    }
}
