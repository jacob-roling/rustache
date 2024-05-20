use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Vec(Vec<Value>),
    Bool(bool),
    Object(HashMap<String, Value>),
    Lambda(fn(current_context: &Value) -> Value),
    None,
}

impl Value {
    fn to_string(&self, context: &Value) -> String {
        return match self {
            Value::Bool(bool) => bool.to_string(),
            Value::Lambda(lambda) => lambda(context).to_string(context),
            Value::String(string) => string.to_string(),
            Value::Vec(array) => array.iter().map(|v| v.to_string(context)).collect(),
            Value::None => "".into(),
            Value::Object(_) => "".into(),
        };
    }

    fn to_bool(&self, context: &Value) -> bool {
        return match self {
            Value::Bool(bool) => *bool,
            Value::Lambda(lambda) => return lambda(context).to_bool(context),
            Value::String(string) => string.len() > 0,
            Value::Vec(array) => array.len() > 0,
            Value::Object(_) => true,
            Value::None => false,
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
        context: &Value,
        partials: Option<&HashMap<String, Vec<Node>>>,
    ) -> Result<(), RenderError>;
}

impl Renderable for &Vec<Node> {
    fn render(
        self,
        writable: &mut impl std::io::Write,
        context: &Value,
        partials: Option<&HashMap<String, Vec<Node>>>,
    ) -> Result<(), RenderError> {
        for node in self {
            if let Err(error) = node.render(writable, context, partials) {
                return Err(error);
            }
        }
        return Ok(());
    }
}

impl Renderable for &Node {
    fn render(
        self,
        writable: &mut impl std::io::Write,
        context: &Value,
        partials: Option<&HashMap<String, Vec<Node>>>,
    ) -> Result<(), RenderError> {
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
                None => return Err(RenderError::IdentifierDoesNotExist(identifier.into())),
            },
            Node::Comment(_comment) => {}
            Node::Section {
                identifier,
                inverted,
                children,
            } => match lookup(identifier.to_string(), context) {
                Some(value) => {
                    if value.to_bool(context) || *inverted {
                        match value {
                            Value::Vec(vec) => {
                                for value in vec {
                                    if let Err(error) = children.render(writable, value, partials) {
                                        return Err(error);
                                    }
                                }
                            }
                            _ => {
                                for child in children {
                                    if let Err(error) = child.render(writable, value, partials) {
                                        return Err(error);
                                    }
                                }
                            }
                        }
                    }
                }
                None => return Err(RenderError::IdentifierDoesNotExist(identifier.into())),
            },
            Node::Implicit => {
                writable
                    .write(context.to_string(context).as_bytes())
                    .unwrap();
            }
            Node::Partial {
                identifier,
                dynamic,
            } => {
                if let Some(partials) = partials {
                    if *dynamic {
                        if let Some(Value::String(dynamic_identifier)) = lookup(identifier.to_string(), context) {
                            if let Some(partial) = partials.get(dynamic_identifier) {
                                if let Err(error) = partial.render(writable, context, Some(partials)) {
                                    return Err(error);
                                }
                            } else {
                                return Err(RenderError::PartialDoesNotExist(identifier.into()));
                            }
                        } else {
                            return Err(RenderError::IdentifierDoesNotExist(identifier.into()));
                        }
                    } else if let Some(partial) = partials.get(identifier) {
                        if let Err(error) = partial.render(writable, context, Some(partials)) {
                            return Err(error);
                        }
                    } else {
                        return Err(RenderError::PartialDoesNotExist(identifier.into()));
                    }
                } else {
                    return Err(RenderError::PartialDoesNotExist(identifier.into()));
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

                    if *dynamic {
                        if let Some(Value::String(dynamic_identifier)) = lookup(identifier.to_string(), context) {
                            if let Some(parent_partial) = partials.get(dynamic_identifier) {
                                if let Err(error) = parent_partial.render(writable, context, Some(&new_partials)) {
                                    return Err(error);
                                }
                            } else {
                                return Err(RenderError::PartialDoesNotExist(identifier.into()));
                            }
                        } else {
                            return Err(RenderError::IdentifierDoesNotExist(identifier.into()));
                        }
                    } else if let Some(parent_partial) = partials.get(identifier) {
                        if let Err(error) = parent_partial.render(writable, context, Some(&new_partials)) {
                            return Err(error);
                        }
                    } else {
                        return Err(RenderError::PartialDoesNotExist(identifier.into()));
                    }
                } else {
                    return Err(RenderError::PartialDoesNotExist(identifier.into()));
                }
            }
            Node::Block {
                identifier,
                children,
            } => {
                if let Some(partials) = partials {
                    if let Some(partial) = partials.get(identifier) {
                        if let Err(error) = partial.render(writable, context, Some(partials)) {
                            return Err(error);
                        }
                    } else {
                        if let Err(error) = children.render(writable, context, Some(partials)) {
                            return Err(error);
                        }
                    }
                }
            }
        }
        return Ok(());
    }
}

fn lookup(identifier: String, context: &Value) -> Option<&Value> {
    return match context {
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
    };
}
