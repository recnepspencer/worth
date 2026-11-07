use crate::basis_lifecycle::BasisFamily;
use crate::effect_lifecycle::{
    discover_effect_lifecycle_support, evaluate_effect_eligibility, normalize_raw_effect_intent,
    EffectDeferredNeighborFamily, EffectDeferredResiduePosture, EffectEligibilityOutcome,
    EffectFamily, EffectSupportPosture, RawEffectIntent,
};
use crate::workflow::{
    WorkflowAuthorityTargetFamily, WorkflowDeclarationFamily, WorkflowFreshnessPolicy,
    WritebackLoweringInput,
};

use super::support::{
    durable_reload_effect_basis, runtime_workflow_binding, store_backed_effect_basis,
    workflow_request,
};

#[test]
fn deferred_support_discovery_exposes_exact_contract_for_store_backed_writeback() {
    let support =
        discover_effect_lifecycle_support(BasisFamily::StoreBacked, EffectFamily::Writeback)
            .expect("fixture digest must fit");
    let contract = support
        .deferred_contract()
        .expect("store-backed writeback should expose deferred contract");

    assert_eq!(support.posture(), EffectSupportPosture::Deferred);
    assert_eq!(
        contract.neighbor_family(),
        EffectDeferredNeighborFamily::StoreBackedExecutionParity
    );
    assert!(contract.leaves_zero_operational_residue());
    assert_eq!(
        contract.residue_posture(),
        EffectDeferredResiduePosture::ZeroOperationalResidue
    );
    assert!(!contract.contract_for_reporting().is_empty());
}

#[test]
fn deferred_support_discovery_exposes_exact_contract_for_durable_writeback() {
    let support =
        discover_effect_lifecycle_support(BasisFamily::DurableReload, EffectFamily::Writeback)
            .expect("fixture digest must fit");
    let contract = support
        .deferred_contract()
        .expect("durable writeback should expose deferred contract");

    assert_eq!(support.posture(), EffectSupportPosture::Deferred);
    assert_eq!(
        contract.neighbor_family(),
        EffectDeferredNeighborFamily::DurableReplayAndRestartStableEnvelope
    );
    assert!(contract.leaves_zero_operational_residue());
}

#[test]
fn deferred_effect_eligibility_matches_support_contract_and_zero_residue() {
    let normalized = normalize_raw_effect_intent(
        &store_backed_effect_basis(),
        RawEffectIntent::Writeback {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        },
    )
    .expect("store-backed writeback should normalize");
    let support =
        discover_effect_lifecycle_support(BasisFamily::StoreBacked, EffectFamily::Writeback)
            .expect("fixture digest must fit");

    let deferred = match evaluate_effect_eligibility(normalized).expect("fixture digest must fit") {
        EffectEligibilityOutcome::Deferred(deferred) => deferred,
        other => panic!("expected deferred effect, got {other:?}"),
    };

    assert_eq!(
        deferred.deferred_contract(),
        &support
            .deferred_contract()
            .expect("store-backed support should expose deferred contract")
    );
    assert!(deferred
        .deferred_contract()
        .leaves_zero_operational_residue());
    assert_eq!(deferred.counters().lowered_effect_count(), 0);
    assert_eq!(deferred.counters().executed_effect_count(), 0);
    assert_eq!(deferred.counters().effect_execution_width(), 0);
}

#[test]
fn deferred_contracts_diverge_for_intentionally_different_future_neighbors() {
    let store = normalize_raw_effect_intent(
        &store_backed_effect_basis(),
        RawEffectIntent::Writeback {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        },
    )
    .expect("store-backed writeback should normalize");
    let durable = normalize_raw_effect_intent(
        &durable_reload_effect_basis(),
        RawEffectIntent::Writeback {
            binding: runtime_workflow_binding(),
            request: workflow_request(
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: WritebackLoweringInput::projected_state_diff(),
        },
    )
    .expect("durable writeback should normalize");

    let store_contract = match evaluate_effect_eligibility(store).expect("fixture digest must fit")
    {
        EffectEligibilityOutcome::Deferred(deferred) => deferred.deferred_contract().clone(),
        other => panic!("expected store-backed deferred effect, got {other:?}"),
    };
    let durable_contract =
        match evaluate_effect_eligibility(durable).expect("fixture digest must fit") {
            EffectEligibilityOutcome::Deferred(deferred) => deferred.deferred_contract().clone(),
            other => panic!("expected durable deferred effect, got {other:?}"),
        };

    assert_ne!(
        store_contract.contract_for_reporting(),
        durable_contract.contract_for_reporting()
    );
}
