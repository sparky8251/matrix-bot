//! Exposes regexes used in various functions
//!
//! Relevant tests are in a test submodule
//!
//! Tests cover false positives, false negatives, and correct captures

#[cfg(test)]
mod tests;

use lazy_static::lazy_static;
use regex::Regex;

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
        (?:\#)                          # Require one # before a number to signify we are searching github
        ([[:digit:]]+)                  # The number to search issues and pulls for (captured)
    ").unwrap();
}
lazy_static! {
    pub static ref LINK_URL: Regex = Regex::new(
        r"(?x)
        ([[:alpha:]]+)                  # The repo to search against (captured)
        (?:[[[\t\v\f\r ]][:blank:]])*?  # Any amount of whitespace but not \n
        (?:@)                           # Require one @ before the article to link
        ([[:alpha:]]+)                  # The number to search issues and pulls for (captured)
    "
    )
    .unwrap();
}
lazy_static! {
    pub static ref GROUP_PING: Regex = Regex::new(
        r"(?x)
        (?:^|\s+)
        %
        (?:[[\t\v\f\r ][:blank:]])*?   # Any amount of whitespace but not \n
        ([[:alnum:]]+)
    ").unwrap();
}
lazy_static! {
    pub static ref CODE_TAG: Regex = Regex::new(r"(?s)(<code>.*</code>)*").unwrap();
}
lazy_static! {
    pub static ref PRE_TAG: Regex = Regex::new(r"(?s)(<pre>.*</pre>)*").unwrap();
}