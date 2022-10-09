//! Exposes regexes used in various functions
//!
//! Relevant tests are in a test submodule
//!
//! Tests cover false positives, false negatives, and correct captures

#[cfg(test)]
mod tests;

use once_cell::sync::Lazy;
use regex::Regex;

pub static UNIT_CONVERSION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
    r"(?x)
    (?:^|\s+)
    ([+-]?[0-9]+(?:.[0-9]+)?)                   # The number to convert, will only allow 1 period for floating points (captured)
    (?:[[\t\v\f\r ][:blank:]])*?                # Any amount of whitespace but not \n
    ([^\s]?[[:alpha:]]+(?:[/\.][[:alpha:]]+)?)  # The unit to convert from including potential ° and / (captured)
    ").unwrap()
});

pub static GITHUB_SEARCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
    r"(?x)
    ([[:alpha:]-]+)                 # The repo to search against (captured)
    (?:[[[\t\v\f\r ]][:blank:]])*?  # Any amount of whitespace but not \n
    (?:\#)                          # Require one # before a number to signify we are searching github
    ([[:digit:]]+)                  # The number to search issues and pulls for (captured)
    ").unwrap()
});

pub static LINK_URL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
    ([[:alpha:]]+)                  # The repo to search against (captured)
    (?:[[[\t\v\f\r ]][:blank:]])*?  # Any amount of whitespace but not \n
    (?:@)                           # Require one @ before the article to link
    ([[:alpha:]]+)                  # The number to search issues and pulls for (captured)
    ",
    )
    .unwrap()
});

pub static GROUP_PING: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
    (?:^|\s+)
    %                              # Require one % to match for a ping
    (?:[[\t\v\f\r ][:blank:]])*?   # Any amount of whitespace but not \n
    ([[:alnum:]]+)
    ",
    )
    .unwrap()
});

pub static TEXT_EXPANSION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
     (?:^|\s+)
     \$                             # Require one $ to match for a text expansion
     (?:[[\t\v\f\r ][:blank:]])*?   # Any amount of whitespace but not \n
     ([[:alnum:]]+)
    ",
    )
    .unwrap()
});

pub static CODE_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)(<code>.*</code>)*").unwrap());

pub static PRE_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)(<pre>.*</pre>)*").unwrap());

pub static MX_REPLY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)(<mx-reply>.*</mx-reply>)*").unwrap());

pub static PARAGRAPH_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)(</*?p>)*").unwrap());

pub static HTTPS_LINE: Lazy<Regex> = Lazy::new(|| Regex::new("([\"]{1}.+[\"]{1})").unwrap());

pub static FORMATTED_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"(@.+:.+)").unwrap());
