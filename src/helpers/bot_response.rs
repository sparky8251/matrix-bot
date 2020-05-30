use std::fmt;

use super::ConvertedUnit;
use url::Url;

#[derive(Debug, Default)]
pub struct BotResponse {
    conversions: Option<Vec<ConvertedUnit>>,
    gh_results: Option<Vec<Url>>,
    links: Option<Vec<Url>>,
}

impl BotResponse {
    pub fn set_unit_conversions(&mut self, conversions: Vec<ConvertedUnit>) {
        self.conversions = Some(conversions)
    }
    pub fn set_gh_results(&mut self, gh_results: Vec<Url>) {
        self.gh_results = Some(gh_results)
    }
    pub fn set_links(&mut self, links: Vec<Url>) {
        self.links = Some(links)
    }
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
