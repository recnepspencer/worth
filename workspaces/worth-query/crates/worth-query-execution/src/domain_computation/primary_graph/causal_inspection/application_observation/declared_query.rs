use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationOutputDemandSource,
    WorthQueryObservedSource,
};

/// Original native evidence retained by one disclosed application query.
///
/// Construction uses the completed source owner's pairing, rather than
/// accepting a receipt and an independently supplied source. The original
/// parameters, scope selector and source meaning remain owner-retained. This
/// retains no native snapshot or read permission; inspection must freshly
/// admit the installed binding, principal, scope and disclosure.
#[derive(Clone)]
pub struct WorthQueryApplicationQueryObservation {
    pub(in crate::domain_computation::primary_graph::causal_inspection) source:
        WorthQueryObservedSource<()>,
    pub(in crate::domain_computation::primary_graph::causal_inspection) basis:
        WorthQueryApplicationBasisIdentity,
    pub(in crate::domain_computation::primary_graph::causal_inspection) result_count: usize,
}

impl WorthQueryApplicationQueryObservation {
    pub(in crate::domain_computation::primary_graph) fn from_disclosed_source<
        Query,
        QueryResult,
    >(
        completed: &WorthQueryApplicationOutputDemandSource<Query, QueryResult>,
    ) -> Option<Self> {
        let source = completed.result_set_observation()?.source().retyped();
        Some(Self {
            source,
            basis: completed.receipt().basis_identity().clone(),
            result_count: completed.receipt().result_count(),
        })
    }

    /// Releases this receipt's retained input and source evidence. It performs
    /// no query, source operation or native retention traversal.
    pub fn close(self) {
        drop(self);
    }
}

impl PartialEq for WorthQueryApplicationQueryObservation {
    fn eq(&self, other: &Self) -> bool {
        self.basis == other.basis
            && self.result_count == other.result_count
            && self.source.idempotency_identity() == other.source.idempotency_identity()
    }
}

impl Eq for WorthQueryApplicationQueryObservation {}

impl std::fmt::Debug for WorthQueryApplicationQueryObservation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationQueryObservation")
            .finish_non_exhaustive()
    }
}
