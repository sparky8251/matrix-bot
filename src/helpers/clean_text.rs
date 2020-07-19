//! Helper function used to strip supplied text of HTML tags and their contents that should not be picked up in later processing steps

use crate::regex::{CODE_TAG, MX_REPLY, PRE_TAG};

use slog::{trace, Logger};

/// Strips provided text of `code`, `pre`, and `mx-reply` HTML tags and their contents
///
/// Returns the result
pub fn clean_text(text: &str, logger: &Logger) -> String {
    let clean_text = CODE_TAG.replace_all(&text, "");
    trace!(logger, "Cleaned text after code tag is {:?}", clean_text);

    let clean_text = PRE_TAG.replace_all(&clean_text, "");
    trace!(logger, "Cleaned text after pre tag is {:?}", clean_text);

    let clean_text = MX_REPLY.replace_all(&clean_text, "");
    trace!(
        logger,
        "Cleaned text after removing reply is {:?}",
        clean_text
    );

    clean_text.to_string()
}
