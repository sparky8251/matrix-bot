mod docs_link;
mod github_search;
mod spellcheck;
mod unit_conversion;

use docs_link::docs_link;
use github_search::github_search;
use spellcheck::spellcheck;
use unit_conversion::unit_conversion;

use crate::config::{Config, Storage};
use crate::regex::{DOCS_LINK, GITHUB_SEARCH, UNIT_CONVERSION};

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
        #[allow(clippy::needless_return)]
        return Ok(());
    } else if UNIT_CONVERSION.is_match(&text.body)
        && text.relates_to == None
        && !GITHUB_SEARCH.is_match(&text.body)
        && config.enable_unit_conversions
    {
        match &text.format {
            Some(v) => {
                if v != "org.matrix.custom.html" {
                    debug!("Message parsed properly, but format {} is unsupported so no conversion is taking place.", v);
                    #[allow(clippy::needless_return)]
                    return Ok(());
                }
            }
            None => (),
        };
        debug!("Entering commandless unit conversion path");
        unit_conversion(&text, &room_id, &client, storage).await?;
    } else if GITHUB_SEARCH.is_match(&text.body)
        && text.relates_to == None
        && !config.repos.is_empty()
    {
        match &text.format {
            Some(v) => {
                if v != "org.matrix.custom.html" {
                    debug!("Message parsed properly, but format {} is unsupported so no search is taking place.", v);
                    #[allow(clippy::needless_return)]
                    return Ok(());
                }
            }
            None => (),
        };
        debug!("Entering commandless github search path");
        github_search(&text, &room_id, &client, storage, &config, &api_client).await?;
    } else if DOCS_LINK.is_match(&text.body)
        && text.relates_to == None
        && !config.docs.is_empty()
        && !config.linkers.is_empty()
    {
        match &text.format {
            Some(v) => {
                if v != "org.matrix.custom.html" {
                    debug!("Message parsed properly, but format {} is unsupported so no search is taking place.", v);
                    #[allow(clippy::needless_return)]
                    return Ok(());
                }
            }
            None => (),
        };
        debug!("Entering commandless docs linking path");
        docs_link(&text, &room_id, &client, storage, &config).await?;
    } else if storage.correction_time_cooldown(room_id)
        && config.enable_corrections
        && text.relates_to == None
        && !config.correction_exclusion.contains(room_id)
    {
        debug!("Entering spell check path");
        spellcheck(&text, &sender, &room_id, &client, &config, &mut storage).await;
    }
    Ok(()) // Nothing to do, skipping message processing
}
