//! An ordinary resumed row uses the same advance's publication permission.
use super::*;

/// Both forms compare captured input through its actual selected owner.
/// Callers supply a basis or a runtime, never an asserted comparison result.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) enum DecisionInput<
    'a,
    'basis,
    'runtime,
    Schema,
> {
    Selected(&'a SelectedDecisionInput<'basis, 'runtime, Schema>),
    Ordinary {
        runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        branch: WorthQueryProductBranch,
    },
}
impl<Schema: ApplicationSchema + 'static> DecisionInput<'_, '_, '_, Schema> {
    pub(super) fn changed(
        &self,
        input: &AcceptedCurrentCandidate,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, ProducerExecutionStop> {
        match self {
            Self::Selected(selected) => selected.changed(input, admission),
            Self::Ordinary { runtime, branch } => {
                // Only a performed member with captured input needs a comparison.
                // A first attempt never buys a redundant selected observation.
                let (shared, positioned) =
                    super::super::selection::select_required_basis(runtime, *branch, admission)?;
                SelectedDecisionInput {
                    shared: &shared,
                    positioned: &positioned,
                    runtime,
                }
                .changed(input, admission)
            }
        }
    }
}
impl PerformedMembers {
    /// Capture the accepted input of an ordinary publication before its caller
    /// can refresh the Ready. A nested required resume shares this same record.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn capture_ordinary<
        Schema: ApplicationSchema + 'static,
    >(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        completion: &crate::domain_computation::primary_graph::application_output_demand::ReadyCompletion,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        for entry in &mut self.entries {
            admission
                .charge_external_work(entry.comparison_work()?)
                .map_err(admission_denial)?;
            if !entry.names(key) || !entry.performed || !entry.input.is_uncaptured() {
                continue;
            }
            super::super::preclaim_required_settlement_arguments(admission)?;
            let candidate = runtime
                .primary_provider
                .graph
                .output_lineage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .resolve_required_settlement(
                    runtime.runtime.authority_identity().as_u64(),
                    &runtime.installed_schema.binding_identity(),
                    completion,
                    admission,
                )
                .map_err(super::super::required_settlement_denial)?;
            entry.input = CapturedDecisionInput::from_result(candidate);
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn ordinary<
        Schema: ApplicationSchema + 'static,
    >(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        source: WorthQueryObservedSourceEpoch,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        branch: WorthQueryProductBranch,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<PublicationReadiness>, WorthQueryOutputDemandDenial> {
        self.fresh(
            key,
            source,
            &DecisionInput::Ordinary { runtime, branch },
            admission,
        )
        .map(|permission| {
            permission.map(|permission| PublicationReadiness {
                member: permission.member,
            })
        })
        .map_err(|stop| match stop {
            ProducerExecutionStop::RequestAdmissionDenied(rejection) => rejection.into_denial(),
            ProducerExecutionStop::ExecutionStopped(denial) => denial,
            ProducerExecutionStop::LiveOutputNotReused { producer, reason } => {
                ProducerExecutionStop::live_output_not_reused(producer, reason)
            }
        })
    }
}
