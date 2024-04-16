use std::{
    io::{BufReader, BufWriter, Cursor, Write},
    thread,
};

use rustache::{
    lexer::lex,
    node::{Data, Value},
    parser::parse,
};

fn main() {
    // println!("{:#?}", templates("views", "**/*.mustache"));
    // let file = File::open("test.mustache").expect("Failed to open file");
    // let reader = BufReader::with_capacity(64, file);

    let (token_sender, token_reciever) = crossbeam_channel::bounded(2);

    thread::spawn(move || {
        // let input = String::from("{{default_tags}}{{=<% %>=}}<%new_tags%>");
        let input = String::from("Hello {{greeting}}");
        let reader = BufReader::with_capacity(128, Cursor::new(input));
        lex(reader, token_sender);
    });

    let template_handle = thread::spawn(move || {
        return parse(token_reciever);
    });

    match template_handle.join() {
        Ok(result) => match result {
            Ok(node) => {
                let mut result = Vec::new();
                let mut writer = BufWriter::new(&mut result);

                node.render(
                    &mut writer,
                    &Data::from([
                        ("greeting".into(), Value::String("world".into())),
                        ("test_lambda".into(), Value::Lambda(test_lambda)),
                    ]),
                );

                writer.flush().unwrap();

                drop(writer);

                println!("{:#?}", String::from_utf8(result).unwrap());
            }
            Err(e) => {
                println!("{:#?}", e);
            }
        },
        Err(e) => {
            println!("{:#?}", e);
        }
    }
}

fn test_lambda(_current_context: Option<Data>) -> String {
    return String::from("test_lambda");
}
