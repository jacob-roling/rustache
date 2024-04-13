use std::io::{BufWriter, Write};

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
    },
    Parent {
        identifier: String,
        dynamic: bool,
    },
}

impl Node {
    fn render(&self, writer: &mut BufWriter<impl std::io::Write>) {
        match self {
            Node::Root(children) => {
                for i in 0..children.len() {
                    children[i].render(writer);
                }
            }
            Node::Section {
                identifier,
                inverted,
                children,
            } => todo!(),
            Node::Variable {
                identifier,
                escaped,
            } => todo!(),
            Node::Text(_) => todo!(),
            Node::Implicit => todo!(),
            Node::Comment(_) => todo!(),
            Node::Partial {
                identifier,
                dynamic,
            } => todo!(),
            Node::Block { identifier } => todo!(),
            Node::Parent {
                identifier,
                dynamic,
            } => todo!(),
        }
    }
}
