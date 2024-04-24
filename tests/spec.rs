mod common;

#[test]
fn spec() {
    common::setup();
    let result = 2 + 2;
    assert_eq!(result, 4);
}
