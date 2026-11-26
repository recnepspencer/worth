//! The missing or already-held entity key behind a refused session run.
use worth_query_execution::facade::primary_graph::WorthQueryKeyedEntityRefusal;
use worth_query_publication::facade::application_entry::WorthQueryApplicationRequestMutationDenial as Request;

use super::WorthQuerySessionRunDenial;

impl<DomainDenial> WorthQuerySessionRunDenial<DomainDenial> {
    /// The entity type, field and key a `NotFound` or `AlreadyExists` refusal
    /// names: a handler's `resolve_entity` miss, an unresolved scope key, or a
    /// unique-key collision. `None` for every other refusal.
    pub fn keyed_entity(&self) -> Option<&WorthQueryKeyedEntityRefusal> {
        match self {
            Self::Request(Request::Handler(denial)) => denial.keyed_entity(),
            Self::Request(Request::ScopeResolution(denial)) => denial.keyed_entity(),
            Self::Uncommitted(
                worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted::Denied(
                    denial,
                ),
            ) => denial.keyed_entity(),
            _ => None,
        }
    }
}
