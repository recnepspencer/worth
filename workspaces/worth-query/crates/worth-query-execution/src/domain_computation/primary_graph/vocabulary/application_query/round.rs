//! Moved whole from `domain_computation::primary_graph::application_query::derived_view::registry::predicate::preparation::round`: the items below and every impl they own.

#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum PredicateNoticeDisposition {
    Refresh,
    Unavailable,
}
