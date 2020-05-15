#[cfg(test)]
mod tests;

use graphql_client::*;

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/queries/github_schema.graphql",
    query_path = "src/queries/github_issueorpull.graphql",
    response_derives = "Debug"
)]
pub struct IssueOrPull;
