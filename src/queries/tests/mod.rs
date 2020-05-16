mod common;

use super::issue_or_pull::IssueOrPullRepositoryIssueOrPullRequest::{Issue, PullRequest};
use super::*;

use common::load_access_token;

use reqwest::header::{self, HeaderValue};

#[tokio::test]
async fn issue() {
    let access_token = load_access_token();
    let client = reqwest::Client::new();
    let query = IssueOrPull::build_query(issue_or_pull::Variables {
        name: "jellyfin".to_string(),
        owner: "jellyfin".to_string(),
        number: 1234,
    });

    let response = client
        .post("https://api.github.com/graphql")
        .bearer_auth(access_token)
        .header(
            header::USER_AGENT,
            HeaderValue::from_static("jellyfin-matrix-bot/tester"),
        )
        .json(&query)
        .send()
        .await
        .unwrap();
    let response_body: Response<issue_or_pull::ResponseData> = response.json().await.unwrap();
    let response_data = response_body
        .data
        .expect("missing response data")
        .repository
        .expect("missing repository")
        .issue_or_pull_request
        .expect("missing issue or pull request");

    match response_data {
        Issue(v) => assert_eq!("/jellyfin/jellyfin/issues/1234", v.resource_path),
        _ => panic!("Did not get an issue back like expected"),
    }
}

#[tokio::test]
async fn pull() {
    let access_token = load_access_token();
    let client = reqwest::Client::new();
    let query = IssueOrPull::build_query(issue_or_pull::Variables {
        name: "jellyfin".to_string(),
        owner: "jellyfin".to_string(),
        number: 123,
    });

    let response = client
        .post("https://api.github.com/graphql")
        .bearer_auth(access_token)
        .header(
            header::USER_AGENT,
            HeaderValue::from_static("jellyfin-matrix-bot/tester"),
        )
        .json(&query)
        .send()
        .await
        .unwrap();
    let response_body: Response<issue_or_pull::ResponseData> = response.json().await.unwrap();
    let response_data = response_body
        .data
        .expect("missing response data")
        .repository
        .expect("missing repository")
        .issue_or_pull_request
        .expect("missing issue or pull request");

    match response_data {
        PullRequest(v) => assert_eq!("/jellyfin/jellyfin/pull/123", v.resource_path),
        _ => panic!("Did not get a pull back like expected"),
    }
}

#[tokio::test]
async fn not_found() {
    let access_token = load_access_token();
    let client = reqwest::Client::new();
    let query = IssueOrPull::build_query(issue_or_pull::Variables {
        name: "jellyfin".to_string(),
        owner: "jellyfin".to_string(),
        number: 123456,
    });

    let response = client
        .post("https://api.github.com/graphql")
        .bearer_auth(access_token)
        .header(
            header::USER_AGENT,
            HeaderValue::from_static("jellyfin-matrix-bot/tester"),
        )
        .json(&query)
        .send()
        .await
        .unwrap();
    let response_body: Response<issue_or_pull::ResponseData> = response.json().await.unwrap();
    let response_data = response_body.errors.expect("no errors found");

    if response_data.len() != 1 {
        panic!("Expected 1 error and got {}", response_data.len())
    } else {
        for error in response_data {
            assert_eq!(
                "Could not resolve to an issue or pull request with the number of 123456.",
                error.message
            )
        }
    }
}
