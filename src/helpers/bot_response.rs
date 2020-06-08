//! Helper type and associated functions to enable simple response building

use std::fmt;

use super::ConvertedUnit;
use ruma_client::identifiers::UserId;
use url::Url;

use std::collections::HashSet;

#[derive(Debug, Default)]
/// Type representing response data with helper functions. Used tih notice type replies.
pub struct BotResponseNotice {
    /// List of converted units for response building
    conversions: Option<Vec<ConvertedUnit>>,
    /// List of gh search results for response building
    gh_results: Option<Vec<Url>>,
    /// List of link results for response building
    links: Option<Vec<Url>>,
}

#[derive(Debug, Default)]
pub struct BotResponseText {
    /// List of users that will be pinged for response building
    users: Option<HashSet<UserId>>,
}

impl BotResponseNotice {
    /// Sets member conversions with supplied list of ConvertedUnits
    ///
    /// Will overwrite if suppled a second time
    pub fn set_unit_conversions(&mut self, conversions: Vec<ConvertedUnit>) {
        self.conversions = Some(conversions)
    }
    /// Sets member gh_results with supplied list of Urls
    ///
    /// Will overwrite if suppled a second time
    pub fn set_gh_results(&mut self, gh_results: Vec<Url>) {
        self.gh_results = Some(gh_results)
    }
    /// Sets member links with supplied list of Urls
    ///
    /// Will overwrite if suppled a second time
    pub fn set_links(&mut self, links: Vec<Url>) {
        self.links = Some(links)
    }
    /// Returns `true` if any member field is `Some`
    pub fn is_some(&self) -> bool {
        self.conversions.is_some() || self.gh_results.is_some() || self.links.is_some()
    }
}

impl BotResponseText {
    /// Sets member users with supplied list of users
    ///
    /// Will overwrite if supplied a second time
    pub fn set_users(&mut self, users: HashSet<UserId>) {
        self.users = Some(users)
    }
    /// Returns `true` if any member field is `Some`
    pub fn is_some(&self) -> bool {
        self.users.is_some()
    }
    /// Formats users to be in line with the fancy riot style pings
    pub fn format_text(&self) -> Option<String> {
        match &self.users {
            Some(v) => {
                let mut formatted_text = String::new();
                for user in v {
                    formatted_text.push_str("<a href=\"https://matrix.to/#/");
                    formatted_text.push_str(&user.to_string());
                    formatted_text.push_str("\">");
                    formatted_text.push_str(user.localpart());
                    formatted_text.push_str("</a>\n");
                }
                Some(formatted_text)
            },
            None => None
        }
    }
}

impl fmt::Display for BotResponseNotice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut response = String::new();
        match &self.conversions {
            Some(v) => {
                for s in v {
                    response.push_str(&s.to_string());
                    response.push('\n')
                }
            }
            None => (),
        }
        match &self.gh_results {
            Some(v) => {
                for s in v {
                    response.push_str(s.as_str());
                    response.push('\n')
                }
            }
            None => (),
        }
        match &self.links {
            Some(v) => {
                for s in v {
                    response.push_str(s.as_str());
                    response.push('\n')
                }
            }
            None => (),
        }
        let response = response.trim();
        write!(f, "{}", response)
    }
}

impl fmt::Display for BotResponseText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut response = String::new();
        match &self.users {
            Some(v) => {
                for user in v {
                    response.push_str(&user.localpart());
                    response.push(' ')
                }
            }
            None => (),
        }
        let response = response.trim();
        write!(f, "{}", response)
    }
}
