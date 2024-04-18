pub mod lexer;
pub mod node;
pub mod parser;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use anyhow::{Error, Result};
use glob::glob;
use lexer::lex;
use node::{Node, RenderError, Renderable, Value};
use parser::{parse, ParserError};

#[derive(Debug)]
pub struct Rustache {
    pub directory: String,
    pub partials: HashMap<String, Vec<Node>>,
}

impl Rustache {
    pub fn new(directory: &str, glob_pattern: &str) -> Result<Self, Error> {
        let available_threads = std::thread::available_parallelism().unwrap();

        let mut partials = HashMap::new();

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(available_threads.into())
            .build()
            .unwrap();

        let (result_sender, result_reciever) =
            crossbeam_channel::unbounded::<(String, Result<Vec<Node>, ParserError>)>();

        for entry in
            glob(&[directory, "/", glob_pattern].concat()).expect("failed to read glob pattern")
        {
            if let Ok(path) = entry {
                let name = path
                    .with_extension("")
                    .iter()
                    .skip(1)
                    .collect::<PathBuf>()
                    .to_str()
                    .map(|s| s.to_string());

                if name.is_none() {}

                let (token_sender, token_reciever) = crossbeam_channel::bounded::<lexer::Token>(4);

                let file = File::open(path).expect("failed to open file");

                thread_pool.spawn(move || {
                    let reader = BufReader::with_capacity(128, file);
                    lex(reader, token_sender);
                });

                let result_producer = result_sender.clone();

                thread_pool.spawn(move || {
                    // if let Err(_) = result_producer.send((name.unwrap(), parse(token_reciever))) {}
                    result_producer
                        .send((name.unwrap(), parse(token_reciever)))
                        .unwrap();
                });
            }
        }

        drop(result_sender);

        while let Ok((name, Ok(partial))) = result_reciever.recv() {
            partials.insert(name, partial);
        }

        return Ok(Self {
            directory: directory.into(),
            partials,
        });
    }

    pub fn render(
        &self,
        name: &str,
        writable: &mut impl std::io::Write,
        context: Option<&Value>,
    ) -> Option<node::RenderError> {
        if !self.partials.contains_key(name) {
            return Some(RenderError::PartialDoesNotExist(name.into()));
        }
        let partial = self.partials.get(name).unwrap();
        partial.render(writable, context, Some(&self.partials));
        return None;
    }
}
