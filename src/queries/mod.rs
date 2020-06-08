//! Exposes GraphQL queries used by Github API client
//!
//! Relevant tests are in a test submodule
//!
//! Tests cover all known cases of a query
//! but will not cover unexpected responses from Reqwest

#[cfg(test)]
mod tests;

use graphql_client::*;

/// Type that represents URI results from query
/// Cannot be `Url` as the returned URI is not a complete URL
type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/queries/github_schema.graphql",
    query_path = "src/queries/github_issueorpull.graphql",
    response_derives = "Debug"
)]
/// Query struct derived from file github_issueorpull.graphql
///
/// Reference that file for further details on structure composition
pub struct IssueOrPull;
