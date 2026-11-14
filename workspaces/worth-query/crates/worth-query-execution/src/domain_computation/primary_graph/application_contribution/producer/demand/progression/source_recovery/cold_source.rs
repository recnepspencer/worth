//! Readmitted source data joins actual World history before issuing local custody.
use crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial as Denial, WorthQueryOutputDemandDenialKind as Kind,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryRecoveredApplicationCommitReceipt,
};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(super) fn readmit_recorded_output_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        original: &WorthQueryRecoveredApplicationCommitReceipt,
        kind: PreparedOutputRootKind,
        root: &str,
    ) -> Result<(), Denial> {
        let head = original
            .new_product_head()
            .durable()
            .ok_or_else(unavailable)?;
        let support = self.installed_program_support().ok_or_else(unavailable)?;
        let row = support
            .lifecycle()
            .output_sources()
            .into_iter()
            .find(|row| row.commit == head.selected_commit())
            .ok_or_else(unavailable)?;
        if row.root != root
            || row.key != *original.idempotency_key_identity()
            || row.intent != original.recorded_intent()
            || row.branch != head.branch().name()
            || row.lifecycle != head.incarnation().ordinal()
            || row.generation != head.generation().get()
        {
            return Err(Denial::new(
                Kind::ForeignSource,
                "recorded required source differs from its obligation",
            ));
        }
        let observation = self.readmit_output_observation(execution, row.commit, &row.branch)?;
        if observation.reference_generation().get() != row.generation {
            return Err(unavailable());
        }
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_recorded_parts(row.scope.0, row.scope.1, row.scope.2);
        self.runtime.output_demands.readmit_source_custody(
            original.clone(),
            observation,
            scope,
            kind,
        )
    }
}
fn unavailable() -> Denial {
    Denial::new(
        Kind::RetainedBasisUnavailable,
        "required source has no exact admitted World history",
    )
}
