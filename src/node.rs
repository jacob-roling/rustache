use std::{
    collections::HashMap,
    io::{BufWriter, Write},
};

use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f32),
    Array(Vec<Value>),
    Bool(bool),
    Object(HashMap<String, Value>),
    Lambda(fn(current_context: Option<&Value>) -> Value),
}

impl Value {
    fn to_string(&self, context: Option<&Value>) -> String {
        return match self {
            Value::Number(number) => number.to_string(),
            Value::Bool(bool) => bool.to_string(),
            Value::Lambda(lambda) => lambda(context).to_string(context),
            Value::String(string) => string.to_string(),
            Value::Array(array) => array.iter().map(|v| v.to_string(context)).collect(),
            Value::Object(_) => "".into(),
        };
    }

    fn to_bool(&self, context: Option<&Value>) -> bool {
        return match self {
            Value::Number(number) => number > &0.,
            Value::Bool(bool) => *bool,
            Value::Lambda(lambda) => return lambda(context).to_bool(context),
            Value::String(string) => string.len() > 0,
            Value::Array(array) => array.len() > 0,
            Value::Object(_) => true,
        };
    }
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("identifier: {0} does not exist")]
    IdentifierDoesNotExist(String),
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
        context: Option<&Value>,
    ) -> Option<RenderError> {
        match self {
            Node::Root(children) => {
                for i in 0..children.len() {
                    children[i].render(writer, context);
                }
            }
            Node::Text(text) => {
                writer.write(text.as_bytes()).unwrap();
            }
            Node::Variable {
                identifier,
                escaped,
            } => match lookup(identifier.to_string(), context) {
                Some(value) => {
                    let string_value = value.to_string(context);
                    let escaped_value = match escaped {
                        true => html_escape::encode_text(&string_value),
                        false => string_value.into(),
                    };
                    writer.write(escaped_value.as_bytes()).unwrap();
                }
                None => return Some(RenderError::IdentifierDoesNotExist(identifier.to_string())),
            },
            Node::Comment(_comment) => {}
            Node::Section {
                identifier,
                inverted,
                children,
            } => match lookup(identifier.to_string(), context) {
                Some(value) => {
                    if value.to_bool(context) || *inverted {
                        for i in 0..children.len() {
                            children[i].render(writer, Some(value));
                        }
                    }
                }
                None => return Some(RenderError::IdentifierDoesNotExist(identifier.to_string())),
            },
            Node::Implicit => {
                if let Some(context) = context {
                    writer
                        .write(context.to_string(Some(context)).as_bytes())
                        .unwrap();
                }
            }
            _ => {}
        }
        return None;
    }
}

fn lookup(identifier: String, context: Option<&Value>) -> Option<&Value> {
    return match context {
        Some(context) => match context {
            Value::Object(context) => {
                let parts = identifier.split(".").collect::<Vec<&str>>();
                let mut current_context = context;
                let mut value = None;

                for i in 0..parts.len() {
                    let part = parts[i];

                    match current_context.get(part) {
                        Some(new_context) => match new_context {
                            Value::Object(new_context) => {
                                current_context = new_context;
                            }
                            new_value => {
                                value = Some(new_value);
                            }
                        },
                        None => {
                            return None;
                        }
                    }
                }

                return value;
            }
            _ => Some(context),
        },
        None => None,
    };
}
