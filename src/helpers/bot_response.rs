//! Helper type and associated functions to enable simple response building

use std::fmt;

use super::ConvertedUnit;
use url::Url;

#[derive(Debug, Default)]
/// Type representing response data with helper functions
pub struct BotResponse {
    /// List of converted units for response building
    conversions: Option<Vec<ConvertedUnit>>,
    /// List of gh search results for response building
    gh_results: Option<Vec<Url>>,
    /// List of link results for response building
    links: Option<Vec<Url>>,
}

impl BotResponse {
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

impl fmt::Display for BotResponse {
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
