use crate::regex::{CODE_TAG, PRE_TAG};

use log::trace;

pub fn clean_text(text: &String) -> String {
    let clean_text = CODE_TAG.replace_all(&text, "");
    trace!("Cleaned text after code tag is {:?}", clean_text);
    let clean_text = PRE_TAG.replace_all(&clean_text, "");
    trace!("Cleaned text after pre tag is {:?}", clean_text);
    clean_text.to_string()
}
