pub mod lexer;
pub mod node;
pub mod parser;

use std::{fs::File, io::BufReader, sync::mpsc};

use glob::glob;

pub fn templates(base_directory: &str, glob_pattern: &str) -> Vec<Vec<lexer::Token>> {
    let num_threads = std::thread::available_parallelism().unwrap();

    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads.into())
        .build()
        .unwrap();

    let (list_sender, list_reciever) = mpsc::channel::<Vec<lexer::Token>>();

    for entry in
        glob(&[base_directory, "/", glob_pattern].concat()).expect("failed to read glob pattern")
    {
        match entry {
            Ok(path) => {
                let (token_sender, token_reciever) = mpsc::sync_channel::<lexer::Token>(2);

                thread_pool.spawn(move || {
                    let file = File::open(path).expect("Failed to open file");
                    let reader = BufReader::with_capacity(64, file);
                    lexer::lex(reader, token_sender);
                });

                let list_sender_clone = list_sender.clone();
                thread_pool.spawn(move || {
                    let mut tokens = Vec::new();
                    while let Ok(token) = token_reciever.recv() {
                        tokens.push(token);
                    }
                    list_sender_clone
                        .send(tokens)
                        .expect("Failed to send result");
                })
            }
            Err(e) => panic!("{}", e),
        }
    }

    let mut results = Vec::new();

    for result in list_reciever {
        results.push(result);
    }

    return results;
}
