use std::fmt::{Display, Formatter};
use std::time::SystemTime;

use crate::regex::{CODE_TAG, GITHUB_SEARCH, PRE_TAG, UNIT_CONVERSION};
use crate::session::{SavedSession, SearchableRepos};

use anyhow::Result;
use lazy_static::lazy_static;
use log::{debug, error, trace};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::StatusCode;
use ruma_client::{
    api::r0::message::create_message_event,
    events::{
        room::message::{MessageEventContent, NoticeMessageEventContent, TextMessageEventContent},
        EventJson, EventType,
    },
    identifiers::{RoomId, UserId},
    HttpsClient,
};
use uom::si::f64::*;
use uom::si::length::{centimeter, foot, inch, kilometer, meter, mile};
use uom::si::mass::{kilogram, pound};
use uom::si::thermodynamic_temperature::{degree_celsius, degree_fahrenheit};
use uom::si::velocity::{kilometer_per_hour, mile_per_hour};

#[derive(Clone)]
enum SpellCheckKind {
    SpellCheckInsensitive(InsensitiveSpelling),
    SpellCheckSensitive(SensitiveSpelling),
}

#[derive(Clone)]
struct SpellCheck {
    insensitive: Vec<InsensitiveSpelling>,
    sensitive: Vec<SensitiveSpelling>,
}
#[derive(Clone)]
struct InsensitiveSpelling {
    spelling: String,
}

#[derive(Clone)]
struct SensitiveSpelling {
    spelling: String,
}

impl SpellCheck {
    fn new() -> Self {
        SpellCheck {
            insensitive: vec![
                InsensitiveSpelling::from("Jellyfish"),
                InsensitiveSpelling::from("Jelly Fin"),
            ],
            sensitive: vec![
                SensitiveSpelling::from("JellyFin"),
                SensitiveSpelling::from("jellyFin"),
            ],
        }
    }
}

impl From<&str> for InsensitiveSpelling {
    fn from(str: &str) -> Self {
        InsensitiveSpelling {
            spelling: str.to_string(),
        }
    }
}

impl From<&str> for SensitiveSpelling {
    fn from(str: &str) -> Self {
        SensitiveSpelling {
            spelling: str.to_string(),
        }
    }
}

impl Display for InsensitiveSpelling {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.spelling)
    }
}

impl Display for SensitiveSpelling {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.spelling)
    }
}

impl Display for SpellCheckKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SpellCheckKind::SpellCheckInsensitive(v) => write!(f, "{}", v),
            SpellCheckKind::SpellCheckSensitive(v) => write!(f, "{}", v),
        }
    }
}

lazy_static! {
    static ref SPELL_CHECK: SpellCheck = SpellCheck::new();
}

pub(super) async fn no_command_check(
    text: &TextMessageEventContent,
    sender: &UserId,
    room_id: &RoomId,
    client: &HttpsClient,
    session: &mut SavedSession,
    searchable_repos: &SearchableRepos,
    api_client: &reqwest::Client,
) -> Result<()> {
    if sender.localpart() == session.get_username() {
        // do nothing if message is from self
        trace!("Message is from self, doing nothing");
    } else {
        if UNIT_CONVERSION.is_match(&text.body)
            && text.relates_to == None
            && !GITHUB_SEARCH.is_match(&text.body)
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
            debug!("Entering commandless unit conversion check path");
            let mut conversions = Vec::new();
            match &text.formatted_body {
                Some(v) => {
                    let clean_text = CODE_TAG.replace_all(&v, "");
                    trace!("Cleaned text after code tag is {:?}", clean_text);
                    let clean_text = PRE_TAG.replace_all(&clean_text, "");
                    trace!("Cleaned text after pre tag is {:?}", clean_text);
                    if UNIT_CONVERSION.is_match(&clean_text) {
                        for cap in UNIT_CONVERSION.captures_iter(&clean_text.to_lowercase()) {
                            trace!("{:?}", cap);
                            conversions.push((cap[1].to_string(), cap[2].to_string()))
                        }
                    } else {
                        debug!(
                            "There are no remaining matches after cleaning tags. Doing nothing."
                        );
                        return Ok(());
                    }
                }
                None => {
                    for cap in UNIT_CONVERSION.captures_iter(&text.body.to_lowercase()) {
                        conversions.push((cap[1].to_string(), cap[2].to_string()))
                    }
                }
            }
            let conversions = conversions;
            let mut result = String::new();
            for conversion in conversions {
                let (quantity, unit) = conversion;
                let quantity = match quantity.parse::<f64>() {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Quantity unable to be parsed. Error is {:?}", e);
                        return Ok(());
                    }
                };
                macro_rules! convert_unit {
                        (
                            $unit:expr, $quantity:expr,
                            $(
                                $unit_ty:ident {
                                    $( ( $from_str:expr, $to_str:expr, $from_ty:ty, $to_ty:ty ) ),*
                                    $(,)?
                                }
                            )*
                            _ => {
                                $($default_code:tt)*
                            }
                        ) => {
                            match $unit {
                                $(
                                    $(
                                        $from_str => {
                                            let unit_value = $unit_ty::new::<$from_ty>($quantity);
                                            let converted_quantity = unit_value.get::<$to_ty>();
                                            let from_string = format!("{:.2}{} => ", $quantity, $from_str);
                                            let to_string = format!("{:.2}{}\n", converted_quantity, $to_str);
                                            result.push_str(from_string.as_str());
                                            result.push_str(to_string.as_str());
                                        }
                                    )*
                                )*
                                _ => {
                                    $($default_code)*
                                }
                            }
                        }
                    }
                convert_unit!(unit.as_str(), quantity,
                    Length {
                        ("cm", "in", centimeter, inch),
                        ("m", "ft", meter, foot),
                        ("km", "mi", kilometer, mile),
                        ("in", "cm", inch, centimeter),
                        ("ft", "m", foot, meter),
                        ("mi", "km", mile, kilometer),
                        ("mile", "km", mile, kilometer),
                        ("miles", "km", mile, kilometer),
                    }
                    ThermodynamicTemperature {
                        ("c", "f", degree_celsius, degree_fahrenheit),
                        ("°c", "°f", degree_celsius, degree_fahrenheit),
                        ("f", "c", degree_fahrenheit, degree_celsius),
                        ("°f", "°c", degree_fahrenheit, degree_celsius),
                    }
                    Mass {
                        ("kg", "lbs", kilogram, pound),
                        ("lbs", "kg", pound, kilogram),
                    }
                    Velocity {
                        ("km/h", "mph", kilometer_per_hour, mile_per_hour),
                        ("kmh", "mph", kilometer_per_hour, mile_per_hour),
                        ("kph", "mph", kilometer_per_hour, mile_per_hour),
                        ("kmph", "mph", kilometer_per_hour, mile_per_hour),
                        ("mph", "km/h", mile_per_hour, kilometer_per_hour),
                    }
                    _ => {
                        debug!(
                        "Attempted unknown conversion for unit {:?}",
                        unit.trim().to_lowercase());
                    }
                )
            }

            if result.trim().to_string() != "" {
                let response = client
                    .request(create_message_event::Request {
                        room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
                        event_type: EventType::RoomMessage,
                        txn_id: session.next_txn_id(),
                        data: EventJson::from(MessageEventContent::Notice(
                            NoticeMessageEventContent {
                                body: result.trim().to_string(),
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
        } else if GITHUB_SEARCH.is_match(&text.body) && text.relates_to == None {
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
            let mut searches = Vec::new();
            match &text.formatted_body {
                Some(v) => {
                    let clean_text = CODE_TAG.replace_all(&v, "");
                    trace!("Cleaned text after code tag is {:?}", clean_text);
                    let clean_text = PRE_TAG.replace_all(&clean_text, "");
                    trace!("Cleaned text after pre tag is {:?}", clean_text);
                    if GITHUB_SEARCH.is_match(&clean_text) {
                        for cap in GITHUB_SEARCH.captures_iter(&clean_text.to_lowercase()) {
                            trace!("{:?}", cap);
                            searches.push((cap[1].to_string(), cap[2].to_string()))
                        }
                    } else {
                        debug!(
                            "There are no remaining matches after cleaning tags. Doing nothing."
                        );
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
                if searchable_repos
                    .get_searchable_repos()
                    .contains_key(&repo.to_lowercase())
                {
                    let repo = match searchable_repos
                        .get_searchable_repos()
                        .get(&repo.to_lowercase())
                    {
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
                        .basic_auth(session.get_gh_username(), Some(session.get_gh_password()))
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
                                            txn_id: session.next_txn_id(),
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
                                        .basic_auth(
                                            session.get_gh_username(),
                                            Some(session.get_gh_password()),
                                        )
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
                                                            txn_id: session.next_txn_id(),
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
                                                            txn_id: session.next_txn_id(),
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
                                            error!(
                                                "Unable to search url {} because of error {:?}",
                                                url, e
                                            );
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
        } else {
            if session.correction_time_cooldown(room_id)
                && text.relates_to == None
                && room_id != "!YjAUNWwLVbCthyFrkz:bonifacelabs.ca"
            {
                debug!("Entering spell check path");
                for incorrect_spelling in SPELL_CHECK
                    .insensitive
                    .clone() //FIXME: There has to be a better way than to clone here
                    .into_iter()
                    .map(SpellCheckKind::SpellCheckInsensitive)
                    .chain(
                        SPELL_CHECK
                            .sensitive
                            .clone() //FIXME: There has to be a better way than to clone here
                            .into_iter()
                            .map(SpellCheckKind::SpellCheckSensitive),
                    )
                {
                    let incorrect = match incorrect_spelling.clone() {
                        SpellCheckKind::SpellCheckInsensitive(v) => text
                            .body
                            .to_lowercase()
                            .contains(&v.to_string().to_lowercase()),
                        SpellCheckKind::SpellCheckSensitive(v) => {
                            text.body.contains(&v.to_string())
                        }
                    };
                    if incorrect {
                        let response = client
                            .request(create_message_event::Request {
                                room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
                                event_type: EventType::RoomMessage,
                                txn_id: session.next_txn_id(),
                                data: EventJson::from(MessageEventContent::Text(
                                    TextMessageEventContent {
                                        body: correct_spelling(
                                            sender.localpart(),
                                            &incorrect_spelling.to_string(),
                                        ),
                                        format: None,
                                        formatted_body: None,
                                        relates_to: None,
                                    },
                                )),
                            })
                            .await;
                        match response {
                            Ok(_) => {
                                session.set_last_correction_time(room_id, SystemTime::now());
                                return Ok(());
                            }
                            Err(e) => {
                                error!("{:?}", e);
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }
    return Ok(()); // Nothing to do, skipping response
}

fn correct_spelling(user: &str, incorrect_spelling: &str) -> String {
    format!("I'd just like to interject for a moment {}. What you're referring to as {}, is in fact, Jellyfin, or as I've recently taken to calling it, Emby plus Jellyfin. Jellyfin is not a media server unto itself, but a free component of a media server as defined by Luke Pulverenti. Through a peculiar turn of events, the version of Jellyfin which is widely used today is basically developed with slave labor. Please recognize the harm caused to the slaves by misnaming the project.", user, incorrect_spelling)
}
