use rustache::Rustache;
use serde::Serialize;

#[derive(Serialize)]
struct MyData {
  name: String,
  activity: String,
  address: String,
  smell: String
}

fn main() {
  let mut stdout = std::io::stdout();
  let rustache = Rustache::new("views", "**/*.mustache").expect("failed to parse template files");
  rustache.render("lily", &mut stdout, &MyData{ name: "lily".into(), activity: "skip".into(), address: "disney world".into(), smell: "rose petals".into() }).expect("failed to render template");
}