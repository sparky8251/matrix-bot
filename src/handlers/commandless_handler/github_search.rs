use crate::helpers::clean_text;
use crate::regex::GITHUB_SEARCH;
use crate::{Config, Storage};

use anyhow::Result;
use log::{debug, error, trace};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::StatusCode;
use ruma_client::{
    api::r0::message::create_message_event,
    events::{
        room::message::{MessageEventContent, NoticeMessageEventContent, TextMessageEventContent},
        EventJson, EventType,
    },
    identifiers::RoomId,
    HttpsClient,
};

pub async fn github_search(
    text: &TextMessageEventContent,
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
    config: &Config,
    api_client: &reqwest::Client,
) -> Result<()> {
    let mut searches = Vec::new();
    match &text.formatted_body {
        Some(v) => {
            let clean_text = clean_text(v);
            if GITHUB_SEARCH.is_match(&clean_text) {
                for cap in GITHUB_SEARCH.captures_iter(&clean_text.to_lowercase()) {
                    trace!("{:?}", cap);
                    searches.push((cap[1].to_string(), cap[2].to_string()))
                }
            } else {
                debug!("There are no remaining matches after cleaning tags. Doing nothing.");
                return Ok(());
            }
        }
        None => {
            for cap in GITHUB_SEARCH.captures_iter(&text.body.to_lowercase()) {
                searches.push((cap[1].to_string(), cap[2].to_string()))
            }
        }
    }
    let searches = searches;
    for (repo, number) in searches {
        if config.repos.contains_key(&repo.to_lowercase()) {
            let repo = match config.repos.get(&repo.to_lowercase()) {
                Some(v) => v,
                None => {
                    debug!("Somehow lost repo in between matching and searching.");
                    return Ok(());
                }
            };
            let url = format!(
                "https://api.github.com/repos/{}/{}/{}",
                repo, "issues", number
            );
            let mut headers = HeaderMap::new();
            headers.insert(
                header::ACCEPT,
                HeaderValue::from_static("application/vnd.github.v3+json"),
            );
            headers.insert(
                header::USER_AGENT,
                HeaderValue::from_static("jellyfin-matrix-bot/0.1.0"),
            );
            let headers = headers;
            trace!("Issues search url is {}", url);
            match api_client
                .get(&url)
                .basic_auth(config.gh_uname.clone(), Some(config.gh_pass.clone()))
                .headers(headers.clone())
                .send()
                .await
            {
                Ok(v) => {
                    match v.status() {
                        StatusCode::OK => {
                            let result =
                                format!("https://github.com/{}/{}/{}", repo, "issues", number);
                            let response = client
                                .request(create_message_event::Request {
                                    room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
                                    event_type: EventType::RoomMessage,
                                    txn_id: storage.next_txn_id(),
                                    data: EventJson::from(MessageEventContent::Notice(
                                        NoticeMessageEventContent {
                                            body: result,
                                            relates_to: None,
                                        },
                                    )),
                                })
                                .await;
                            match response {
                                Ok(_) => return Ok(()),
                                Err(e) => {
                                    error!("{:?}", e);
                                    return Ok(());
                                }
                            }
                        }
                        StatusCode::NOT_FOUND => {
                            let url = format!(
                                "https://api.github.com/repos/{}/{}/{}",
                                repo, "pulls", number
                            );
                            trace!("Pulls search url is {}", url);
                            match api_client
                                .get(&url)
                                .basic_auth(config.gh_uname.clone(), Some(config.gh_pass.clone()))
                                .headers(headers.clone())
                                .send()
                                .await
                            {
                                Ok(v) => {
                                    match v.status() {
                                        StatusCode::OK => {
                                            let result = format!(
                                                "https://github.com/{}/{}/{}",
                                                repo, "issues", number
                                            );
                                            let response = client
                                                .request(create_message_event::Request {
                                                    room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
                                                    event_type: EventType::RoomMessage,
                                                    txn_id: storage.next_txn_id(),
                                                    data: EventJson::from(
                                                        MessageEventContent::Notice(
                                                            NoticeMessageEventContent {
                                                                body: result,
                                                                relates_to: None,
                                                            },
                                                        ),
                                                    ),
                                                })
                                                .await;
                                            match response {
                                                Ok(_) => return Ok(()),
                                                Err(e) => {
                                                    error!("{:?}", e);
                                                    return Ok(());
                                                }
                                            }
                                        }
                                        StatusCode::NOT_FOUND => {
                                            let result = format!(
                                                "Unable to find issue or pull for {}#{}",
                                                repo, number
                                            );
                                            let response = client
                                                .request(create_message_event::Request {
                                                    room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
                                                    event_type: EventType::RoomMessage,
                                                    txn_id: storage.next_txn_id(),
                                                    data: EventJson::from(
                                                        MessageEventContent::Notice(
                                                            NoticeMessageEventContent {
                                                                body: result,
                                                                relates_to: None,
                                                            },
                                                        ),
                                                    ),
                                                })
                                                .await;
                                            match response {
                                                Ok(_) => return Ok(()),
                                                Err(e) => {
                                                    error!("{:?}", e);
                                                    return Ok(());
                                                }
                                            }
                                        }
                                        _ => {
                                            error!(
                                                "Unexpected status code {:?}. {:?}",
                                                v.status(),
                                                v
                                            );
                                            return Ok(());
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Unable to search url {} because of error {:?}", url, e);
                                    return Ok(());
                                }
                            }
                        }
                        _ => {
                            error!("Unexpected status code {:?}. {:?}", v.status(), v);
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    error!("Unable to search url {} because of error {:?}", url, e);
                    return Ok(());
                }
            }
        } else {
            trace!("Repo not found in searchable repo list.");
            return Ok(());
        }
    }
    return Ok(()); // Nothing done, return
}
