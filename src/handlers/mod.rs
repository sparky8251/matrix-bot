mod correct_spelling;
mod helpers;
// mod roll;
mod unit_conversion;

use self::correct_spelling::correct_spelling_check;
use self::helpers::do_nothing;
#[allow(unused_imports)]
// use self::roll::roll;
use self::unit_conversion::unit_conversion;
use crate::session::SavedSession;

use anyhow::Result;
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use ruma_client::{
    events::room::message::TextMessageEventContent,
    identifiers::{RoomId, UserId},
    HttpsClient,
};

lazy_static! {
    static ref NO_BANG: Regex = Regex::new(r"(^[^!])").unwrap();
}
lazy_static! {
    static ref UNIT_CONVERSION: Regex = Regex::new(
        r"(?x)
        ^!convert                       # The tag from line start
        \s*?                            # Any amount of whitespace
        ([[:digit:]]+\.[[:digit:]]+)    # The number to convert (captured)
        \s*?                            # Any amount of white space
        ([[:alpha:]/]+)                 # The unit to convert from including potential / (captured)
    "
    )
    .unwrap();
}
lazy_static! {
    // TODO: fix regex so it will match '!roll 22 90' just as well as '!roll 22' but never '!roll 22 90 120'
    static ref ROLL: Regex = Regex::new(r"(^!roll\s*?[[:digit]][\s+?[[:digit:]]]x{0,1}?)").unwrap();
}

pub async fn handle_text_message(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
) -> Result<()> {
    if NO_BANG.is_match(&text.body) {
        debug!("Entering spell check path...");
        correct_spelling_check(text, sender, room_id, client, session).await
    } else if UNIT_CONVERSION.is_match(&text.body.to_lowercase()) {
        debug!("Entering unit conversion path...");
        unit_conversion(text, room_id, client, session).await
    } else if ROLL.is_match(&text.body.to_lowercase()) {
        debug!("Entering roll path...");
        // roll(text, room_id, client, session).await
        do_nothing().await
    } else {
        debug!("Entering do nothing path...");
        do_nothing().await
    }
}
