use rustache::Rustache;
use serde::Serialize;

#[derive(Serialize)]
struct MyData {
  greeting: String,
  partial: String,
  parent: String
}

fn main() {
  let mut stdout = std::io::stdout();
  let rustache = Rustache::new("views", "**/*.mustache").expect("failed to parse template files");
  rustache.render("test", &mut stdout, &MyData{ greeting: "world".into(), partial: "partials/footer".into(), parent: "layouts/base".into() }).expect("failed to render template");
}