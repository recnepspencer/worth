//! Live receiver access, never retained by the original interpretation token.
use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::program_occurrence::OriginalProgramMeaning;
use crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory;
use crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial;
use std::sync::{Arc, Weak};
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn bind_original_meaning_support(
        &self,
        support: &Arc<QueryObligationHistory>,
    ) -> Result<(), QueryObligationDenial> {
        let weak = Arc::downgrade(support);
        let actual = self.original_meaning_support.get_or_init(|| weak.clone());
        if !Weak::ptr_eq(actual, &weak) {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::provider) fn acknowledge_original_meaning(
        &self,
        meaning: Option<&OriginalProgramMeaning>,
        opened: &worth_runtime_world::facade::WorldOpenedPublicationAttempt,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        let Some(meaning) = meaning else {
            return Ok(());
        };
        let support = self
            .original_meaning_support
            .get()
            .and_then(Weak::upgrade)
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        meaning.acknowledge_membership(&support, opened, request)
    }
}
