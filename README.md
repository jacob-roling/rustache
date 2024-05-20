# Rustache

Fast, simple and fully spec compliant implementation of the Mustache template engine in Rust.

## Usage

`views/hello.mustache`

```html
<h1>Hello {{ greeting }}</h1>
```

```rust
use rustache::Rustache;
use serde::Serialize;

#[derive(Serialize)]
struct MyData {
  greeting: String
}

fn main() {
  let mut stdout = std::io::stdout();
  let mut rustache = Rustache::new("views", "**/*.mustache").unwrap();
  rustache.render("hello", &MyData{ greeting: "world".into() }).unwrap();
}
```
