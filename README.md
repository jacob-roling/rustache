# Rustache

Fast, simple and specification compliant\* implementation of the Mustache template engine in Rust.

See the [Mustache Specification](https://github.com/mustache/spec).

Supported features:

- [x] Variables
- [x] Dotted Names
- [x] Implicit Operator
- [x] Lambdas
- [x] Sections
- [x] Non-Empty Lists
- [x] Non-False Values
- [x] Inverted Sections
- [x] Comments
- [x] Partials
- [x] Blocks
- [x] Set Delimiter
- [x] Dynamic Variables
- [x] Dynamic Partials
- [x] Dynamic Parents
- [ ] Lambdas That Return Templates

> [!IMPORTANT]  
> \*Lambdas returning templates cannot be type checked therefore this implementation differs from the spec in this regard by instead passing the current context to lambda calls. This effectively enables the same result to be achieved but with type safety.

## Usage

`views/hello.mustache`

```html
<h1>Hello {{ greeting }}</h1>
```

`src/main.rs`

```rust
use rustache::Rustache;
use serde::Serialize;

#[derive(Serialize)]
struct MyData {
  greeting: String
}

fn main() {
  let mut stdout = std::io::stdout();
  let rustache = Rustache::new("views", "**/*.mustache").expect("failed to parse template(s)");
  rustache.render("hello", &mut stdout, &MyData{ greeting: "Rustache!".into() }).expect("failed to render template");
}
```

Result:

```html
<h1>Hello Rustache!</h1>
```
