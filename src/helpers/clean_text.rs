//! Helper function used to strip supplied text of HTML tags and their contents that should not be picked up in later processing steps

use crate::regex::{CODE_TAG, PRE_TAG};

use log::trace;

/// Strips provided text of `code` and `pre` HTML tags and their contents
///
/// Returns the result
pub fn clean_text(text: &str) -> String {
    let clean_text = CODE_TAG.replace_all(&text, "");
    trace!("Cleaned text after code tag is {:?}", clean_text);

    let clean_text = PRE_TAG.replace_all(&clean_text, "");
    trace!("Cleaned text after pre tag is {:?}", clean_text);

    clean_text.to_string()
}
