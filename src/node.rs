use std::collections::HashMap;
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
    #[error("identifier: '{0}' does not exist")]
    IdentifierDoesNotExist(String),
    #[error("partial: '{0}' does not exist")]
    PartialDoesNotExist(String),
}

#[derive(Debug, Clone)]
pub enum Node {
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

pub trait Renderable {
    fn render(
        self,
        writable: &mut impl std::io::Write,
        context: Option<&Value>,
        partials: Option<&HashMap<String, Vec<Node>>>,
    ) -> Option<RenderError>;
}

impl Renderable for &Vec<Node> {
    fn render(
        self,
        writable: &mut impl std::io::Write,
        context: Option<&Value>,
        partials: Option<&HashMap<String, Vec<Node>>>,
    ) -> Option<RenderError> {
        for node in self {
            if let Some(error) = node.render(writable, context, partials) {
                return Some(error);
            }
        }
        return None;
    }
}

impl Renderable for &Node {
    fn render(
        self,
        writable: &mut impl std::io::Write,
        context: Option<&Value>,
        partials: Option<&HashMap<String, Vec<Node>>>,
    ) -> Option<RenderError> {
        match self {
            Node::Text(text) => {
                writable.write(text.as_bytes()).unwrap();
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
                    writable.write(escaped_value.as_bytes()).unwrap();
                }
                None => return Some(RenderError::IdentifierDoesNotExist(identifier.into())),
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
                            if let Some(error) = children[i].render(writable, Some(value), partials)
                            {
                                return Some(error);
                            }
                        }
                    }
                }
                None => return Some(RenderError::IdentifierDoesNotExist(identifier.into())),
            },
            Node::Implicit => {
                if let Some(context) = context {
                    writable
                        .write(context.to_string(Some(context)).as_bytes())
                        .unwrap();
                }
            }
            Node::Partial {
                identifier,
                dynamic,
            } => {
                if let Some(partials) = partials {
                    if let Some(partial) = partials.get(identifier) {
                        if let Some(error) = partial.render(writable, context, Some(partials)) {
                            return Some(error);
                        }
                    } else {
                        return Some(RenderError::PartialDoesNotExist(identifier.into()));
                    }
                } else {
                    return Some(RenderError::PartialDoesNotExist(identifier.into()));
                }
            }
            Node::Parent {
                identifier,
                dynamic,
                children,
            } => {
                if let Some(partials) = partials {
                    let mut new_partials = partials.clone();

                    for node in children {
                        if let Node::Block {
                            identifier,
                            children,
                        } = node
                        {
                            new_partials.insert(identifier.into(), children.clone());
                        }
                    }

                    if let Some(parent_partial) = partials.get(identifier) {
                        parent_partial.render(writable, context, Some(&new_partials));
                    } else {
                        return Some(RenderError::PartialDoesNotExist(identifier.into()));
                    }
                } else {
                    return Some(RenderError::PartialDoesNotExist(identifier.into()));
                }
            }
            Node::Block {
                identifier,
                children,
            } => {
                if let Some(partials) = partials {
                    if let Some(partial) = partials.get(identifier) {
                        partial.render(writable, context, Some(partials));
                    } else {
                        children.render(writable, context, Some(partials));
                    }
                }
            }
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
