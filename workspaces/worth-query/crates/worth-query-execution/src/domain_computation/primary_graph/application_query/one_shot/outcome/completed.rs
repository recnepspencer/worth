//! Native query completion before selecting its public result or source closure.
use super::*;

pub(in crate::domain_computation::primary_graph::application_query) struct CompletedOneShotRead<
    Query,
    QueryResult,
> {
    pub(in crate::domain_computation::primary_graph::application_query) read:
        WorthQueryApplicationOneShotResult<Query, QueryResult>,
    pub(in crate::domain_computation::primary_graph::application_query) authorization:
        crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts,
}
