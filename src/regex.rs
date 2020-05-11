use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref NO_BANG: Regex = Regex::new(r"(^[^!])").unwrap();
}
lazy_static! {
    pub static ref SINGLE_UNIT_CONVERSION: Regex = Regex::new(
        r"(?x)
        ^!convert                           # The tag from line start
        [[:space:][:blank:]]*?              # Any amount of whitespace
        ([[:digit:]]+\.{0,1}[[:digit:]]*)   # The number to convert, will only allow 1 period for floating points (captured)
        [[:space:][:blank:]]*?              # Any amount of white space
        ([[:alpha:]/]+)                     # The unit to convert from including potential / (captured)
    "
    )
    .unwrap();
}
lazy_static! {
    pub static ref UNIT_CONVERSION: Regex = Regex::new(
        r"(?x)
        ([[:digit:]]+\.{0,1}[[:digit:]]*)   # The number to convert, will only allow 1 period for floating points (captured)
        [[:space:][:blank:]]*?              # Any amount of white space
        ([[:alpha:]/]+)                     # The unit to convert from including potential / (captured)
    "
    )
    .unwrap();
}
lazy_static! {
    // TODO: fix regex so it will match '!roll 22 90' just as well as '!roll 22' but never '!roll 22 90 120'
    pub static ref ROLL: Regex = Regex::new(r"(^!roll\s*?[[:digit]][\s+?[[:digit:]]]x{0,1}?)").unwrap();
}
