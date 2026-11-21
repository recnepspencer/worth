//! Exact component obligations survive a cold checkpoint without eager history.
//! Restored C2 portions of
//! 0615dd733c:crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:
//! cold residency of the selected and unrelated envelope cells, full physical
//! demand of both original roots, and the retained-root roster census.
//! Fixture adaptations: the same-SHA `original` fixture is the testkit's
//! `original` over `TestDurableMedium` (it records the unrelated commit too);
//! cell residency comes from the owner's `observe_commit_envelope_custody`; the
//! envelope demand is the public replay read of the same cell; the private
//! `in_canonical_frame` descriptor comparison is the public recorded export of
//! the same basis; full root demand and its read snapshot are
//! `RelationalBasisCustodyWitness::read_retained_full`; the private directory
//! decode is the owner's `inspect_component_roster`. The unrelated-artifact
//! deletion belongs to C4 and is not restored here.
//! `recorded_original_retirement_preserves_both_leases_across_busy_and_stop`
//! needs a boundary ruling (see the C2 capture progress note): its fixture
//! holds the owner's private external cut and lane gate.
use super::*;
use std::sync::Arc;
use worth_relational::facade::{
    durability::RelationalDurableColdSource, history::CommitId, runtime::RelationalRuntime,
};

fn resident(runtime: &RelationalRuntime, commit: CommitId) -> bool {
    runtime
        .observe_commit_envelope_custody()
        .into_iter()
        .find(|cell| cell.commit_id == commit)
        .expect("retained commit cell")
        .resident
}

fn original_custody(empty: bool) {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, empty);
    let offers = original.medium.offered().len();
    let (runtime, authority) = recover(execution, &original.medium);
    let subject = runtime
        .durability_authority()
        .discover_associated_effects(execution, &original.association)
        .unwrap()
        .pop()
        .unwrap();
    assert!(!resident(&runtime, subject.receipt().commit_id));
    let effect = runtime
        .durability_authority()
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    let selected = runtime
        .replay()
        .canonical_commit_envelope(execution, subject.receipt().commit_id)
        .unwrap()
        .unwrap();
    let semantic = selected.semantic_result();
    assert_eq!(effect.semantic_result(), semantic);
    let proof = authority
        .admit_recorded_publication(
            execution,
            &runtime,
            effect,
            &original.before,
            &original.after,
        )
        .unwrap();
    assert_eq!(proof.custody().effect().semantic_result(), semantic);
    // The same shared Native owner performs maintenance while real custody
    // pins preserve both original roots.
    let shared: &RelationalRuntime = &runtime;
    shared.run_branch_root_reclamation_pass();
    for (basis, recorded) in [
        (proof.custody().predecessor(), &original.before),
        (proof.custody().successor(), &original.after),
    ] {
        let exported = runtime
            .owner_component_services()
            .basis_port()
            .export_recorded_retained_component(execution, basis)
            .unwrap();
        assert_eq!(exported.descriptor(), recorded.descriptor());
        let witness = basis.custody_witness();
        assert_eq!(
            witness.read_retained_full(execution, &runtime).unwrap(),
            Some(witness.schema_authority_digest())
        );
    }
    assert!(runtime
        .owner_component_services()
        .basis_port()
        .compare_current_exact(proof.custody().successor())
        .is_err());
    assert_eq!(
        original.medium.offered().len(),
        offers,
        "recovery never republishes"
    );
    assert!(!resident(&runtime, original.unrelated_id));
    drop(proof);
    runtime.prepare_close().unwrap().finish();
}

#[test]
fn recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor() {
    original_custody(false);
}

#[test]
fn recorded_component_checkpoint_readmits_original_empty_predecessor() {
    original_custody(true);
}

#[test]
fn recorded_component_checkpoint_omits_released_components_and_plain_observations() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, false);
    let source: Arc<dyn RelationalDurableColdSource> = original.medium.clone();
    let checkpoint = capture(execution, &original.source);
    original.medium.file_checkpoint(checkpoint.clone());
    let roster = checkpoint
        .inspect_component_roster(execution, Some(source.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(roster.retained_roots, 1);
    assert_eq!(roster.retained_empty_components, 0);
    assert!(!roster
        .recorded_components
        .iter()
        .any(|component| component == original.before.descriptor()
            || component == original.after.descriptor()));

    // An ordinary observation has no external durable component obligation.
    let (_, observed) = original
        .source
        .observe_branch(&original.source.main_branch_identity())
        .unwrap();
    create_entity(
        execution,
        &original.source,
        "observation-alone-is-not-a-component",
    );
    let next = capture(execution, &original.source);
    original.medium.file_checkpoint(next.clone());
    let roster = next
        .inspect_component_roster(execution, Some(source))
        .unwrap()
        .unwrap();
    assert_eq!(roster.retained_roots, 1);
    drop(observed);
}
