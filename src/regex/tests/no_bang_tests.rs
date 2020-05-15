use crate::regex::*;

#[test]
fn test_bang_at_front() {
    assert_eq!(false, NO_BANG.is_match("!convert"));
}
#[test]
fn test_bang_at_end() {
    assert_eq!(true, NO_BANG.is_match("convert!"));
}
#[test]
fn test_bang_after_starting_space() {
    assert_eq!(true, NO_BANG.is_match(" !convert"));
}
#[test]
fn test_no_bang() {
    assert_eq!(true, NO_BANG.is_match("hoogaboogawooga"));
}
