//! Performs search of issues and pulls in message text and builds proper response

use crate::config::Config;
use crate::helpers::{clean_text, BotResponse};
use crate::queries::issue_or_pull::IssueOrPullRepositoryIssueOrPullRequest::{Issue, PullRequest};
use crate::queries::*;
use crate::regex::GITHUB_SEARCH;

use graphql_client::GraphQLQuery;
use log::{debug, error, trace};
use reqwest::header;
use ruma_client::events::room::message::TextMessageEventContent;
use url::Url;

/// Searches and links found issues or pulls requested and builds response text
pub async fn github_search(
    text: &TextMessageEventContent,
    config: &Config,
    api_client: &reqwest::Client,
    response: &mut BotResponse,
) {
    let mut repos_to_search = Vec::new();
    match &text.formatted_body {
        Some(v) => {
            let clean_text = clean_text(v);
            if GITHUB_SEARCH.is_match(&clean_text) {
                for cap in GITHUB_SEARCH.captures_iter(&clean_text.to_lowercase()) {
                    trace!("{:?}", cap);
                    repos_to_search.push((cap[1].to_string(), cap[2].to_string()))
                }
            } else {
                debug!("There are no remaining matches after cleaning tags. Doing nothing.");
                return;
            }
        }
        None => {
            for cap in GITHUB_SEARCH.captures_iter(&text.body.to_lowercase()) {
                repos_to_search.push((cap[1].to_string(), cap[2].to_string()))
            }
        }
    }
    let repos_to_search = repos_to_search;
    let mut searches = Vec::new();
    for (repo, number) in repos_to_search {
        match number.parse::<i64>() {
            Ok(n) => match config.repos.get(&repo.to_lowercase()) {
                Some(r) => {
                    let index = match r.find('/') {
                        Some(v) => v,
                        None => {
                            debug!("No / was found in repo/owner pair {:?}. Unable to search such a thing.", r);
                            continue;
                        }
                    };
                    let (owner, repo) = r.split_at(index);
                    let repo = repo.replace('/', "");
                    searches.push((owner.to_string(), repo.to_string(), n))
                }
                None => {
                    debug!("Repo {:?} not found", repo);
                    continue;
                }
            },
            Err(e) => {
                error!(
                    "Issue or pull number unable to be parsed. Error is {:?}, quantity is {:?}",
                    e, number
                );
            }
        }
    }
    let searches = searches;
    debug!("Queued searches: {:?}", searches);
    if searches.is_empty() {
        debug!("No searches found after parsing numbers. No searches will be built.");
        return;
    }
    let mut results = Vec::new();
    for (owner, name, number) in searches {
        let query = IssueOrPull::build_query(issue_or_pull::Variables {
            name,
            owner,
            number,
        });
        let response_body = match api_client
            .post("https://api.github.com/graphql")
            .bearer_auth(config.gh_access_token.clone())
            .header(header::USER_AGENT, config.user_agent.clone())
            .json(&query)
            .send()
            .await
        {
            Ok(r) => {
                let response_body: graphql_client::Response<issue_or_pull::ResponseData> =
                    match r.json().await {
                        Ok(b) => b,
                        Err(e) => {
                            error!("No response body found. Error is {:?}", e);
                            continue;
                        }
                    };
                response_body
            }
            Err(e) => {
                error!("Query failed, Error is {:?}", e);
                continue;
            }
        };
        let response_data = match response_body.data {
            Some(d) => match d.repository {
                Some(r) => match r.issue_or_pull_request {
                    Some(v) => v,
                    None => {
                        error!("Missing issue or pull request data");
                        continue;
                    }
                },
                None => {
                    error!("Missing repository data");
                    continue;
                }
            },
            None => {
                error!("Missing response data");
                continue;
            }
        };

        match response_data {
            Issue(v) => {
                let result = "https://github.com".to_string() + &v.resource_path + "\n";
                match Url::parse(&result) {
                    Ok(v) => results.push(v),
                    Err(e) => error!(
                        "Unable to parse result {:?} to Url due to error {:?}",
                        result, e
                    ),
                }
            }
            PullRequest(v) => {
                let result = "https://github.com".to_string() + &v.resource_path + "\n";
                match Url::parse(&result) {
                    Ok(v) => results.push(v),
                    Err(e) => error!(
                        "Unable to parse result {:?} to Url due to error {:?}",
                        result, e
                    ),
                }
            }
        }
    }
    if results.is_empty() {
        error!("No search resulted returned. Doing nothing");
    } else {
        response.set_gh_results(results)
    }
    // match client
    //     .request(create_message_event::Request {
    //         room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
    //         event_type: EventType::RoomMessage,
    //         txn_id: storage.next_txn_id(),
    //         data: EventJson::from(MessageEventContent::Notice(NoticeMessageEventContent {
    //             body: results,
    //             relates_to: None,
    //             format: None,
    //             formatted_body: None,
    //         }))
    //         .into_json(),
    //     })
    //     .await
    // {
    //     Ok(_) => Ok(()),
    //     Err(e) => {
    //         error!("{:?}", e);
    //         Ok(())
    //     }
    // }
}
