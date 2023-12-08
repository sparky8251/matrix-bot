//! Helper type and associated functions to enable simple response building

use super::ConvertedUnit;
use reqwest::Url;
use ruma::OwnedUserId;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Default)]
/// Type representing response data with helper functions. Used tih notice type replies.
pub struct MatrixNoticeResponse {
    /// List of converted units for response building
    conversions: Option<Vec<ConvertedUnit>>,
    /// List of gh search results for response building
    gh_results: Option<Vec<Url>>,
    /// List of link results for response building
    links: Option<Vec<Url>>,
    /// Expanded text for response building
    expanded_text: Option<Vec<String>>,
}

#[derive(Debug, Default)]
pub struct MatrixFormattedTextResponse {
    /// List of users that will be pinged for response building
    users: Option<HashSet<OwnedUserId>>,
}

#[derive(Debug, Default)]
pub struct MatrixFormattedNoticeResponse {
    errors: Option<Vec<String>>,
}

impl MatrixNoticeResponse {
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
    /// Sets member expanded text with supplied text
    ///
    /// Will overwrite if suppled a second time
    pub fn set_expanded_text(&mut self, expanded_text: Vec<String>) {
        self.expanded_text = Some(expanded_text)
    }
    /// Returns `true` if any member field is `Some`
    pub fn is_some(&self) -> bool {
        self.conversions.is_some()
            || self.gh_results.is_some()
            || self.links.is_some()
            || self.expanded_text.is_some()
    }
}

impl MatrixFormattedTextResponse {
    /// Sets member users with supplied list of users
    ///
    /// Will overwrite if supplied a second time
    pub fn set_users(&mut self, users: HashSet<OwnedUserId>) {
        self.users = Some(users)
    }
    /// Returns `true` if any member field is `Some`
    pub fn is_some(&self) -> bool {
        self.users.is_some()
    }
    /// Formats users to be in line with the fancy riot style pings
    pub fn format_text(&self) -> Option<String> {
        self.users.as_ref().map(|v| {
            let mut formatted_text = String::new();
            for user in v {
                formatted_text.push_str("<a href=\"https://matrix.to/#/");
                formatted_text.push_str(user.as_ref());
                formatted_text.push_str("\">");
                formatted_text.push_str(user.localpart());
                formatted_text.push_str("</a>\n");
            }
            formatted_text
        })
    }
}

impl MatrixFormattedNoticeResponse {
    pub fn add_errrors(&mut self, errors: Vec<String>) {
        match &mut self.errors {
            Some(v) => {
                for e in errors {
                    v.push(e)
                }
            }
            None => self.errors = Some(errors),
        }
    }
    // pub fn is_some(&self) -> bool {
    //     self.errors.is_some()
    // }
    pub fn format_text(&self) -> Option<String> {
        self.errors.as_ref().map(|v| {
            let mut formatted_text = String::new();
            for error in v {
                formatted_text.push_str("<font color=\"#ff4b55\">");
                formatted_text.push_str(error);
                formatted_text.push_str("</font>\n")
            }
            formatted_text
        })
    }
}

impl fmt::Display for MatrixNoticeResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut response = String::new();
        if let Some(v) = &self.conversions {
            for s in v {
                response.push_str(&s.to_string());
                response.push('\n')
            }
        }
        if let Some(v) = &self.gh_results {
            for s in v {
                response.push_str(s.as_ref());
                response.push('\n')
            }
        }
        if let Some(v) = &self.links {
            for s in v {
                response.push_str(s.as_ref());
                response.push('\n')
            }
        }
        if let Some(v) = &self.expanded_text {
            for s in v {
                response.push_str(s);
                response.push('\n')
            }
        }
        let response = response.trim();
        f.write_str(response)
    }
}

impl fmt::Display for MatrixFormattedTextResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut response = String::new();
        if let Some(v) = &self.users {
            for user in v {
                response.push_str(user.localpart());
                response.push(' ')
            }
        }
        let response = response.trim();
        f.write_str(response)
    }
}

impl fmt::Display for MatrixFormattedNoticeResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut response = String::new();
        if let Some(v) = &self.errors {
            for error in v {
                response.push_str(error);
                response.push('\n')
            }
        }
        let response = response.trim();
        f.write_str(response)
    }
}
