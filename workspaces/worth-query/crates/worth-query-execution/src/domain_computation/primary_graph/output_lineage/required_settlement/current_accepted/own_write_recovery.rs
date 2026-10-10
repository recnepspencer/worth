//! Registration recovery must preserve the original computation's own-write staleness.
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::{
        WorthQueryAcceptedOutputAuthority, WorthQueryCompletedOutputDemand,
        WorthQueryOutputReadinessDeliveryEvidence,
    },
    output_lineage::input_cutoff::RetainedInputCutoffCandidate,
    tests::{application_attempt::resolved_account, fixture::live_scope},
};
use std::sync::Arc;

#[test]
fn registration_recovery_does_not_certify_an_output_computed_before_its_own_write() {
    check_own_write_recovery(false);
}

#[test]
fn accepted_current_cannot_bypass_own_write_evidence_after_index_loss() {
    check_own_write_recovery(true);
}

fn check_own_write_recovery(index_loss: bool) {
    crate::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write(|world, receipt, cell, resources| {
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let retained_product = world.selected_product();
    let retained_snapshot = handle.with_runtime(|runtime| {
        let basis = runtime.admit_branch_basis(&runtime.main_branch_identity()).unwrap();
        runtime.snapshots().snapshot_for_observation(&basis.observation()).unwrap()
    });
    let maximum = resources.installation().maximum_retained_bytes;
    let completed = world.application.primary_provider.observe_completed_application(receipt.commit_reference()).unwrap();
    let recorded = cell.get().unwrap();
    if index_loss {
        crate::domain_computation::primary_graph::output_lineage::own_write_fixture::evict_index_with_unrelated_write(world, &resources);
    } else {
    // Release only the derived row, preserving the actual performed lineage.
    // Re-registering under an exhausted ledger now follows the genuine stop
    // path; no fact, witness or authority is manufactured by this fixture.
    assert_eq!(
        owner.retire_settlements(
            std::slice::from_ref(&recorded.settlement_identity),
            &mut owner.edit_admission()
        ),
        vec![Arc::clone(&recorded.settlement_identity)]
    );
    let held = resources
        .reserve_retained_capacity(maximum - resources.retained_capacity_bytes())
        .unwrap();
    handle.with_runtime(|runtime| {
        let basis = runtime.admit_branch_basis(&runtime.main_branch_identity()).unwrap();
        let snapshot = runtime.snapshots().snapshot_for_observation(&basis.observation()).unwrap();
        let reason = crate::domain_computation::primary_graph::output_lineage::invalidation::register_completed(
            owner, &completed, recorded.computation_source, &[], runtime, &snapshot, Arc::clone(&recorded.settlement_identity), None,
            recorded.native_output_witness(), &mut owner.edit_admission()).unwrap_err();
        assert_eq!(reason, FullVerificationReason::RegistrationIncomplete);
        recorded.require_verification(reason);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
    drop(held);
    }
    assert!(recorded.observed_source_facts().unwrap().for_comparison().is_none(),
        "accepted-current cannot obtain comparable postconditions of a stale computation");
    let selected = RetainedInputCutoffCandidate::from_exact_cell(cell);
    let ready = ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
        authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
        readiness: WorthQueryOutputReadinessDeliveryEvidence::for_test(),
        resources: None,
    });
    let candidate = AcceptedCurrentCandidate { selected, ready };
    let (_, product, _) = retained_product.into_parts();
    handle.with_runtime(|runtime| {
        let snapshot = retained_snapshot;
        let selected = runtime.read_truth().positioned_snapshot(&snapshot).unwrap();
        assert!(candidate
            .decision_input_changed_at(owner, runtime, &snapshot, &selected, &mut owner.edit_admission())
            .unwrap(),
            "the performed own write is evidence of a changed decision input");
        let answer = candidate
            .certify_current(
                owner,
                runtime,
                product.read_lease_ref(),
                &snapshot,
                &selected,
                &mut owner.edit_admission(),
            )
            .unwrap();
        assert!(
            matches!(answer, CurrentAcceptedResult::NeedsDisclosure),
            "the output computed from x=1 cannot be Current after its own write of x=2"
        );
        drop(answer);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
    resolved_account(world, "2", &live_scope());
    });
}

#[test]
fn checkpoint_excludes_facts_of_output_computed_before_its_own_write() {
    crate::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write(|_, _, cell, _| {
        assert!(cell.get().unwrap().checkpoint_source_facts().is_none());
    });
}
