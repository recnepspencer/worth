//! Semantic capture seals precede installed realized-proof issuance.
use crate::domain_computation::primary_graph::access_strategy::coverage::RetainedCoverage;
mod candidate_rows;
mod capture;
mod revalidation;
use super::{adapter::InstalledIndexProvider, predicate::RetainedPredicateProbe};
use crate::domain_computation::{
    primary_graph::source_facts::SourceFactsAllocation,
    provider_session::WorthQueryCompletedSourceRead,
};
use candidate_rows::CapturedCandidateRows;
pub(super) use capture::{capture, CaptureInput};
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_relational::facade::runtime::ManagedIndexSourceObservation;

struct CapturedIndexInfluence<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> {
    adapter: Arc<dyn InstalledIndexProvider<S, I>>,
    original: Arc<ManagedIndexSourceObservation>,
    scope: worth_relational::facade::identity::EntityId,
    parameters: Arc<RetainedPredicateProbe>,
    candidates: CapturedCandidateRows,
    _coverage: Arc<RetainedCoverage>,
    _admitted_source: Arc<WorthQueryCompletedSourceRead>,
    _allocation: Arc<SourceFactsAllocation>,
}

/// Only this owner and its capture/revalidation children can construct this stage
/// after checking actual admitted containment and all genuine candidate bodies.
pub(in crate::domain_computation::primary_graph) struct CheckedManagedFootprintCapture<'a> {
    frame: worth_proof::BorrowedRealizedFootprintFrame<'a>,
    coverage: &'a RetainedCoverage,
}
impl CheckedManagedFootprintCapture<'_> {
    pub(in crate::domain_computation::primary_graph) fn frame(
        &self,
    ) -> worth_proof::BorrowedRealizedFootprintFrame<'_> {
        self.frame
    }
    pub(in crate::domain_computation::primary_graph) fn coverage(&self) -> &RetainedCoverage {
        self.coverage
    }
}
