//! Delay actual selection after original lease loss, then reclaim real roots.
//! Restored from 22e638d638:crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs
//! (`delayed_selection` and both variants). Fixture adaptations: the private
//! `Weak` upgrades of the basis and its root become the owner's
//! `RelationalBasisCustodyWitness`; the upgraded full-root demand and read
//! snapshot become `read_retained_full`, which performs the same demand and
//! snapshot through the same retained observation and returns its schema digest.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial;
use worth_testkit_relational::durable_publication::{create_entity, durable_runtime};

fn delayed_selection(empty: bool) {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    if !empty {
        create_entity(execution, &runtime, "original-selected-component");
    }
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let original = basis.custody_witness();
    let original_schema = original.schema_authority_digest();
    let lease = runtime.retain_component_basis(&basis).unwrap();
    drop(basis);
    create_entity(execution, &runtime, "later-current-head");
    let checkpoint = owned_capture(execution, &runtime, &medium, &[]);
    let observer = checkpoint.clone();
    runtime.release_component_basis(lease).unwrap();
    runtime.run_branch_root_reclamation_pass();
    assert!(
        original.root_retained(),
        "actual capture keeps the selected immutable root before filing"
    );
    assert!(
        original.basis_retained(),
        "the same actual observation survives original lease release"
    );
    assert_eq!(
        original.read_retained_full(execution, &runtime).unwrap(),
        Some(original_schema)
    );
    medium.file_checkpoint(checkpoint.clone());
    // Original selection is now actual. Interrupt the caller only after it
    // enters its one real execution scope and begins Native acknowledgement.
    let cancellation = CancellationSource::new();
    let backing =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let stopped_request = ExecutionRequest::serial(&backing);
    let mut entered = false;
    let _interrupted = stopped_request.run_native_with_checkpoint(|context| {
        entered = true;
        cancellation.cancel();
        let result = checkpoint.acknowledge_selected(&stopped_request, context);
        assert!(matches!(
            result,
            Err(RelationalCheckpointAcknowledgementDenial::Stopped(_))
        ));
        result
    });
    assert!(entered);
    drop(checkpoint);
    runtime.run_branch_root_reclamation_pass();
    assert!(
        original.basis_retained(),
        "partial acknowledgement and an old observer drop cannot retire the capture custody"
    );
    acknowledge(execution, &observer);
    assert!(!original.basis_retained(), "successful original acknowledgement retires shared capture read custody even while observers remain");
    runtime.run_branch_root_reclamation_pass();
    assert!(
        !original.root_retained(),
        "retired root is now reclaimable through the actual owner"
    );
    runtime.prepare_close().unwrap().finish();
    drop(observer);
}
#[test]
fn native_custody_correction_delayed_canonical_selection_retains_actual_root_and_interrupted_ack() {
    delayed_selection(false);
}
#[test]
fn native_custody_correction_delayed_empty_selection_retains_actual_schema_and_interrupted_ack() {
    delayed_selection(true);
}
