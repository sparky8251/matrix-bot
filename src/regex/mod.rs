#[cfg(test)]
mod tests;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref NO_BANG: Regex = Regex::new(r"(^[^!])").unwrap();
}
lazy_static! {
    pub static ref UNIT_CONVERSION_COMMAND: Regex = Regex::new(
        r"(?x)
        ^!convert                                   # The tag from line start
    ").unwrap();
}
lazy_static! {
    pub static ref UNIT_CONVERSION: Regex = Regex::new(
        r"(?x)
        (?:^|\s+)
        ([+-]?[0-9]+(?:.[0-9]+)?)                   # The number to convert, will only allow 1 period for floating points (captured)
        (?:[[\t\v\f\r ][:blank:]])*?                # Any amount of whitespace but not \n
        ([^\s]?[[:alpha:]]+(?:[/\.][[:alpha:]]+)?)  # The unit to convert from including potential ° and / (captured)
    ").unwrap();
}
lazy_static! {
    pub static ref GITHUB_SEARCH: Regex = Regex::new(
        r"(?x)
        ([[:alpha:]-]+)                 # The repo to search against (captured)
        (?:[[[\t\v\f\r ]][:blank:]])*?  # Any amount of whitespace but not \n
        (?:\#)                          # Required one # before a number to signify we are searching github
        ([[:digit:]]+)                  # The number to search issues and pulls for (captured)
    ").unwrap();
}
lazy_static! {
    pub static ref CODE_TAG: Regex = Regex::new(r"(?s)(<code>.*</code>)*").unwrap();
}
lazy_static! {
    pub static ref PRE_TAG: Regex = Regex::new(r"(?s)(<pre>.*</pre>)*").unwrap();
}
lazy_static! {
    // TODO: fix regex so it will match '!roll 22 90' just as well as '!roll 22' but never '!roll 22 90 120'
    pub static ref ROLL: Regex = Regex::new(r"(^!roll\s*?[[:digit]][\s+?[[:digit:]]]x{0,1}?)").unwrap();
}
