use rustache::{EmptyContext, Rustache};
use serde::Serialize;
use std::io::Write;
use may_minihttp::{HttpServer, HttpService, Request, Response};

#[derive(Debug, Serialize)]
struct Index {
    greeting: String,
}

#[derive(Clone)]
struct HelloWorld {
    rustache: &Rustache
};

impl HttpService for HelloWorld {
    fn call(&mut self, _req: Request, res: &mut Response) -> io::Result<()> {
        if let Some(error) = self.rustache.render(
            "index",
            res.writer,
            &Index {
                greeting: "Test".into(),
            },
        ) {
            println!("{}", error);
        }
        Ok(())
    }
}

fn main() {
    let server = HttpServer(HelloWorld {
        rustache: &Rustache::new("views", "**/*.mustache").unwrap()
    }).start("0.0.0.0:8080").unwrap();

    server.join().unwrap();

    // let mut stdout = std::io::stdout();
    // stdout.flush().unwrap();
}
