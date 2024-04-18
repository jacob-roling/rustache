use std::{
    collections::HashMap,
    io::{BufReader, BufWriter, Cursor, Write},
    thread,
};

use rustache::{lexer::lex, node::Value, parser::parse};

fn main() {
    // println!("{:#?}", templates("views", "**/*.mustache"));
    // let file = File::open("test.mustache").expect("Failed to open file");
    // let reader = BufReader::with_capacity(64, file);

    let (token_sender, token_reciever) = crossbeam_channel::bounded(2);

    thread::spawn(move || {
        // let input = String::from("{{default_tags}}{{=<% %>=}}<%new_tags%>");
        let input = String::from(
            "{{<article}}
  Never shown
  {{$body}}
    {{#headlines}}
    <p>{{.}}</p>
    {{/headlines}}
  {{/body}}
{{/article}}

{{<article}}
  {{$title}}Yesterday{{/title}}
{{/article}}",
        );
        // let input = String::from("Hello {{placeholder}} asdjksand");

        let reader = BufReader::with_capacity(128, Cursor::new(input));
        lex(reader, token_sender);
    });

    let template_handle = thread::spawn(move || {
        return parse(token_reciever);
    });

    match template_handle
        .join()
        .expect("couldn't join on the associated thread")
    {
        Ok(nodes) => {
            println!("{:#?}", nodes);

            // let mut result = Vec::new();
            // let mut writer = BufWriter::new(&mut result);

            // for node in nodes {
            //     node.render(
            //         &mut writer,
            //         Some(&Value::Object(HashMap::<String, Value>::from([(
            //             "placeholder".into(),
            //             Value::String("christmas".into()),
            //         )]))),
            //         None,
            //     );
            // }

            // writer.flush().unwrap();

            // drop(writer);

            // println!("{:#?}", String::from_utf8(result).unwrap());
        }
        Err(e) => println!("{}", e),
    };
}
