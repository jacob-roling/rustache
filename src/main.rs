use std::{
    collections::HashMap,
    io::{BufReader, BufWriter, Cursor, Write},
    thread,
};

use rustache::{lexer::lex, node::Value, _old_parser::parse};

fn main() {
    // println!("{:#?}", templates("views", "**/*.mustache"));
    // let file = File::open("test.mustache").expect("Failed to open file");
    // let reader = BufReader::with_capacity(64, file);

    let (token_sender, token_reciever) = crossbeam_channel::bounded(2);

    thread::spawn(move || {
        // let input = String::from("{{default_tags}}{{=<% %>=}}<%new_tags%>");
        let input = String::from("{{#section}}{{.}}{{/section}}");
        let reader = BufReader::with_capacity(128, Cursor::new(input));
        lex(reader, token_sender);
    });

    let template_handle = thread::spawn(move || {
        return parse(token_reciever);
    });

    match template_handle
        .join()
        .expect("Couldn't join on the associated thread")
    {
        Ok(node) => {
            println!("{:#?}", node);
            // let mut result = Vec::new();
            // let mut writer = BufWriter::new(&mut result);

            // node.render(
            //     &mut writer,
            //     &Value::Object(HashMap::<String, Value>::from([(
            //         "greeting".into(),
            //         Value::String("world".into()),
            //     )])),
            // );

            // writer.flush().unwrap();

            // drop(writer);

            // println!("{:#?}", String::from_utf8(result).unwrap());
        }
        Err(e) => println!("{:#?}", e),
    };
}
