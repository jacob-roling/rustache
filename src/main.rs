use rustache::Rustache;

fn main() {
    let rustache = Rustache::new("views", "**/*.mustache").unwrap();
    let mut stdout = std::io::stdout().lock();
    println!("{:#?}", rustache);
    rustache.render("test", &mut stdout, None);
}
