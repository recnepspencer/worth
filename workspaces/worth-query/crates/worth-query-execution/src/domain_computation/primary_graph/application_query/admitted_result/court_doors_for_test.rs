//! Sealed test doors over this module's private members.
use super::*;

impl<Query, QueryResult> WorthQueryApplicationOutputDemandSource<Query, QueryResult> {
    #[doc(hidden)]
    pub fn into_single_source_for_test(
        self) -> Option<(QueryResult, super::super::WorthQueryObservedSource<Query>)> {
        Self::into_single_source(self)
    }
}
