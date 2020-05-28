mod link_url;
mod github_search;
mod spellcheck;
mod unit_conversion;

use link_url::link_url;
use github_search::github_search;
use spellcheck::spellcheck;
use unit_conversion::unit_conversion;

use crate::config::{Config, Storage};
use crate::helpers::check_format;
use crate::regex::{LINK_URL, GITHUB_SEARCH, UNIT_CONVERSION};

use anyhow::Result;
use log::{debug, error, trace};
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
    } else if UNIT_CONVERSION.is_match(&text.body)
        && text.relates_to.is_none()
        && !GITHUB_SEARCH.is_match(&text.body)
        && config.enable_unit_conversions
    {
        match check_format(&text.format) {
            Ok(_) => {
                debug!("Entering commandless unit conversion path");
                unit_conversion(&text, &room_id, &client, storage).await?
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    } else if GITHUB_SEARCH.is_match(&text.body)
        && text.relates_to.is_none()
        && !config.repos.is_empty()
    {
        match check_format(&text.format) {
            Ok(_) => {
                debug!("Entering commandless github search path");
                github_search(&text, &room_id, &client, storage, &config, &api_client).await?
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    } else if LINK_URL.is_match(&text.body)
        && text.relates_to.is_none()
        && !config.links.is_empty()
        && !config.linkers.is_empty()
    {
        match check_format(&text.format) {
            Ok(_) => {
                debug!("Entering commandless url linking path");
                link_url(&text, &room_id, &client, storage, &config).await?
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    } else if storage.correction_time_cooldown(room_id)
        && config.enable_corrections
        && text.relates_to.is_none()
        && !config.correction_exclusion.contains(room_id)
    {
        debug!("Entering spell check path");
        spellcheck(&text, &sender, &room_id, &client, &config, &mut storage).await;
    }
    Ok(()) // Nothing to do, skipping message processing
}
