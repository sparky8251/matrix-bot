mod github_search;
mod spellcheck;
mod unit_conversion;

use github_search::github_search;
use spellcheck::spellcheck;
use unit_conversion::unit_conversion;

use crate::regex::{GITHUB_SEARCH, UNIT_CONVERSION};
use crate::{Config, Storage};

use anyhow::Result;
use log::{debug, trace};
use ruma_client::{
    events::room::message::TextMessageEventContent,
    identifiers::{RoomId, UserId},
    HttpsClient,
};

pub(super) async fn commandless_handler(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    mut storage: &mut Storage,
    config: &Config,
    api_client: &reqwest::Client,
) -> Result<()> {
    if sender == &config.mx_uname {
        // do nothing if message is from self
        trace!("Message is from self, doing nothing");
        return Ok(());
    } else {
        if UNIT_CONVERSION.is_match(&text.body)
            && text.relates_to == None
            && !GITHUB_SEARCH.is_match(&text.body)
            && config.enable_unit_conversions
        {
            match &text.format {
                Some(v) => {
                    if v != "org.matrix.custom.html" {
                        debug!("Message parsed properly, but format {} is unsupported so no conversion is taking place.", v);
                        return Ok(());
                    }
                }
                None => (),
            };
            debug!("Entering commandless unit conversion path");
            unit_conversion(&text, &room_id, &client, storage).await?;
        } else if GITHUB_SEARCH.is_match(&text.body) && text.relates_to == None  && config.repos.len() > 0 {
            match &text.format {
                Some(v) => {
                    if v != "org.matrix.custom.html" {
                        debug!("Message parsed properly, but format {} is unsupported so no search is taking place.", v);
                        return Ok(());
                    }
                }
                None => (),
            };
            debug!("Entering commandless github search path");
            github_search(&text, &room_id, &client, storage, &config, &api_client).await?;
        } else {
            if storage.correction_time_cooldown(room_id) && config.enable_corrections
                && text.relates_to == None
                && room_id != "!YjAUNWwLVbCthyFrkz:bonifacelabs.ca"
            {
                debug!("Entering spell check path");
                spellcheck(&text, &sender, &room_id, &client, &config, &mut storage).await;
            }
        }
        return Ok(()); // Nothing to do, skipping response
    }
}
