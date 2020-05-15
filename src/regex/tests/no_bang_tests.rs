use crate::regex::*;

#[test]
fn at_front() {
    assert_eq!(false, NO_BANG.is_match("!convert"));
}
#[test]
fn at_end() {
    assert_eq!(true, NO_BANG.is_match("convert!"));
}
#[test]
fn after_starting_space() {
    assert_eq!(true, NO_BANG.is_match(" !convert"));
}
#[test]
fn no_bang() {
    assert_eq!(true, NO_BANG.is_match("hoogaboogawooga"));
}
