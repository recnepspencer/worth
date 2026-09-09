use super::{
    bridge_reconstruction_denial, WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryReconstructedTemporalIntent,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_query_installation::facade::WorthQueryTemporalIntentLifecycle;
use worth_runtime_bridge::facade::{
    BridgeManagedClockBinding, BridgeManagedTemporalDenial, BridgeManagedTemporalIntentIdentity,
    BridgeManagedTemporalIntentLifecycle, BridgeManagedTemporalIntentReconciliation,
    BridgeManagedTemporalIntentReconciliationParts, BridgePreparedConditionalReconstitution,
    BridgeSealedRuntimeAssembly,
};

pub(in crate::domain_computation::primary_graph::conditional_operation) fn reconcile_temporal_intents<
    Clock,
    Input,
>(
    bridge: &BridgeSealedRuntimeAssembly,
    clock: &BridgeManagedClockBinding,
    candidates: &mut BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
    reconcile_intents(clock, candidates, |parts| {
        bridge.reconcile_managed_temporal_intent(parts)
    })
}

pub(in crate::domain_computation::primary_graph::conditional_operation) fn reconcile_prepared_temporal_intents<
    Clock,
    Input,
>(
    bridge: &mut BridgePreparedConditionalReconstitution,
    clock: &BridgeManagedClockBinding,
    candidates: &mut BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
    reconcile_intents(clock, candidates, |parts| {
        bridge.reconcile_managed_temporal_intent(parts)
    })
}

fn reconcile_intents<Clock, Input>(
    clock: &BridgeManagedClockBinding,
    candidates: &mut BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
    mut reconcile: impl FnMut(
        BridgeManagedTemporalIntentReconciliationParts<'_>,
    ) -> Result<
        BridgeManagedTemporalIntentReconciliation,
        BridgeManagedTemporalDenial,
    >,
) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
    for reconstructed in candidates.values() {
        let candidate = reconstructed.candidate();
        let identity = BridgeManagedTemporalIntentIdentity::declare(Arc::<str>::from(
            candidate.identity().as_str(),
        ))
        .map_err(|denial| bridge_reconstruction_denial(denial.detail()))?;
        let lifecycle = match candidate.lifecycle() {
            WorthQueryTemporalIntentLifecycle::Active => {
                BridgeManagedTemporalIntentLifecycle::Active
            }
            WorthQueryTemporalIntentLifecycle::Cancelled => {
                BridgeManagedTemporalIntentLifecycle::Cancelled
            }
            WorthQueryTemporalIntentLifecycle::Completed => {
                BridgeManagedTemporalIntentLifecycle::Completed
            }
        };
        let outcome = reconcile(BridgeManagedTemporalIntentReconciliationParts {
            binding: clock,
            identity,
            revision: candidate.revision(),
            due_coordinate: candidate.due().nanoseconds(),
            idempotency_identity: Arc::from(candidate.idempotency().as_str()),
            source_record_identity: reconstructed.source_record(),
            lifecycle,
        })
        .map_err(|denial| bridge_reconstruction_denial(denial.detail()))?;
        let expected = matches!(
            (candidate.lifecycle(), outcome),
            (
                WorthQueryTemporalIntentLifecycle::Active,
                BridgeManagedTemporalIntentReconciliation::Installed
            ) | (
                WorthQueryTemporalIntentLifecycle::Cancelled
                    | WorthQueryTemporalIntentLifecycle::Completed,
                BridgeManagedTemporalIntentReconciliation::TerminalNoop
            )
        );
        if !expected {
            return Err(bridge_reconstruction_denial(
                "fresh conditional publication observed non-fresh temporal intent state",
            ));
        }
    }
    candidates.retain(|_, intent| {
        intent.candidate().lifecycle() == WorthQueryTemporalIntentLifecycle::Active
    });
    Ok(())
}
