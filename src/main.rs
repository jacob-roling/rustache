use rustache::Rustache;
use serde::Serialize;

#[derive(Serialize)]
struct MyData {
  greeting: String
}

fn main() {
  let mut stdout = std::io::stdout();
  let rustache = Rustache::new("views", "**/*.mustache").expect("failed to parse template files");
  rustache.render("hello", &mut stdout, &MyData{ greeting: "world".into() }).expect("failed to render template");
}