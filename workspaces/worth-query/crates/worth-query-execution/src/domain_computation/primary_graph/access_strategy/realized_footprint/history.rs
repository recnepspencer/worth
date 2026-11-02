//! Only admitted retained influence may classify actual canonical managed history.
use super::CapturedManagedInfluence;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::{
        RelationalAuthoredManagedSourceImpact, RelationalAuthoredWriteDenial,
        RelationalAuthoredWriteOccurrence,
    },
    runtime::RelationalRuntime,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum ManagedHistoryRelevanceDenial {
    Native(RelationalAuthoredWriteDenial),
    Provider(crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexWorkDenial),
}
impl CapturedManagedInfluence {
    pub(in crate::domain_computation::primary_graph) fn history_impact(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        occurrence: &RelationalAuthoredWriteOccurrence,
    ) -> Result<RelationalAuthoredManagedSourceImpact, ManagedHistoryRelevanceDenial> {
        execution.consult().map_err(|cause| {
            ManagedHistoryRelevanceDenial::Native(RelationalAuthoredWriteDenial::Request(cause))
        })?;
        if !self.authority.admits(&self.proof) {
            return Err(ManagedHistoryRelevanceDenial::Native(
                RelationalAuthoredWriteDenial::UnsupportedManagedSourceSelector,
            ));
        }
        self.support.history_impact(runtime, execution, occurrence)
    }
    /// The caller has left every Native/runtime/attempt gate.
    pub(in crate::domain_computation::primary_graph) fn history_relevant(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        impact: RelationalAuthoredManagedSourceImpact,
    ) -> Result<bool, ManagedHistoryRelevanceDenial> {
        execution.consult().map_err(|cause| {
            ManagedHistoryRelevanceDenial::Native(RelationalAuthoredWriteDenial::Request(cause))
        })?;
        if !self.authority.admits(&self.proof) {
            return Err(ManagedHistoryRelevanceDenial::Native(
                RelationalAuthoredWriteDenial::UnsupportedManagedSourceSelector,
            ));
        }
        self.support.history_relevant(execution, impact)
    }
}
