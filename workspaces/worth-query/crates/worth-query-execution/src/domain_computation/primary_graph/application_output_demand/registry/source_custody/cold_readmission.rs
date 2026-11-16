//! Attach local runtime custody only after the durable source and World agree.
use super::super::SourceCustody;
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputSourceEvidence, WorthQueryRecoveredApplicationCommitReceipt,
};

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn readmit_source_custody(
        &self,
        original: WorthQueryRecoveredApplicationCommitReceipt,
        observation: worth_runtime_world::facade::ProductBranchObservation,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        root_kind: PreparedOutputRootKind,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let commit = observation.selected_commit().clone();
        let occurrence = observation.lifecycle_incarnation();
        let receipt =
            WorthQueryOutputSourceEvidence::readmitted(original, observation.clone(), scope);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(existing) = state.source_custody.get(&commit) {
            if existing.root_kind != root_kind
                || existing
                    .source
                    .as_ref()
                    .is_none_or(|source| !source.receipt.same_retained_output_source_as(&receipt))
            {
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "source custody already has another root or original",
                ));
            }
            return Ok(());
        }
        state.source_custody.insert(
            commit,
            SourceCustody {
                cleanup: Default::default(),
                occurrence,
                root_kind,
                source: Some(WorthQueryRetainedOutputDemandSource {
                    receipt,
                    change: None,
                    observation,
                    output_source_identity: None,
                }),
                discovery: None,
                bound_sources: None,
                consumed_sources: Vec::new(),
                retired_sources: Vec::new(),
                retired: None,
                token_count: 0,
                completed: false,
            },
        );
        Ok(())
    }
}
