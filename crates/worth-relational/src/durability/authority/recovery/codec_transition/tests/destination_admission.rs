//! Codec restore cannot replace a native destination already visible to a holder.
use super::fixture::*;
use crate::durability::data::{
    RecoveryFailureClass, RelationalNativeCheckpoint, RelationalRecoveryStop,
};
use crate::mvcc::RelationalTransactionIntent;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::WorkerIntentBatch;

fn assert_destination_refused_unchanged(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: &mut RelationalRuntime,
    native: &RelationalNativeCheckpoint,
) {
    let instance = target.runtime_instance_id();
    let schema = target.config().schema.registry;
    let history = target.history().immutable_commit_count();
    let snapshots = target.visibility.active_snapshot_count();
    let checkpoint = capture(target);
    let error = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(execution, native)
        .err()
        .expect("bound destination must be refused");
    let RelationalRecoveryStop::Durability(error) = error else {
        panic!("destination refusal must retain its durability classification")
    };
    assert_eq!(error.class, RecoveryFailureClass::ReplayFailure);
    assert!(
        error.detail.starts_with("codec recovery destination "),
        "{}",
        error.detail
    );
    assert_eq!(target.runtime_instance_id(), instance);
    assert_eq!(target.config().schema.registry, schema);
    assert_eq!(target.history().immutable_commit_count(), history);
    assert_eq!(target.visibility.active_snapshot_count(), snapshots);
    assert_eq!(capture(target), checkpoint);
}

#[test]
fn fresh_destination_publication_binding_refuses_then_unbound_retry_restores() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let source = world(execution, true);
    let native = capture(&source.runtime);
    let (mut target, _sealed_receipt) = installed(2);
    let instance = target.runtime_instance_id();
    let publication = target.publication_port();
    assert_destination_refused_unchanged(execution, &mut target, &native);
    // No destination state was published. Releasing its actual service binding
    // restores exclusive admission while the sealed initial receipt stays held.
    drop(publication);
    let (_, recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(execution, &native)
        .unwrap();
    assert!(recovered.codec_transition_pending());
    assert_ne!(target.runtime_instance_id(), instance);
    assert_eq!(
        protected_payload(&target, source.authored),
        protected_payload(&source.runtime, source.authored)
    );
}

#[test]
fn fresh_destination_preparation_port_remains_usable_and_unbound_retry_restores() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let source = world(execution, true);
    let native = capture(&source.runtime);
    let (mut target, _sealed_receipt) = installed(2);
    let preparation = target.preparation_port();
    assert_destination_refused_unchanged(execution, &mut target, &native);
    {
        let selected = basis(&target);
        let mut transaction = target
            .begin_branch_transaction(&selected, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("held preparation after restore refusal"),
                execution.allocation_policy(),
            )
            .unwrap();
        let candidate = preparation
            .prepare_branch_transaction(execution, transaction)
            .unwrap();
        preparation.discard_prepared_candidate(candidate).unwrap();
        assert!(selected.is_current());
    }
    assert_eq!(target.history().immutable_commit_count(), 0);
    assert_eq!(target.visibility.active_snapshot_count(), 0);
    drop(preparation);
    let (_, recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(execution, &native)
        .unwrap();
    assert!(recovered.codec_transition_pending());
}

#[test]
fn fresh_destination_actual_weak_state_binding_refuses_without_closing_settlement() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let source = world(execution, true);
    let native = capture(&source.runtime);
    let (mut target, _sealed_receipt) = installed(2);
    let settlement = target.settlement_port();
    let instance = settlement.runtime_instance_id();
    assert_destination_refused_unchanged(execution, &mut target, &native);
    assert_eq!(settlement.runtime_instance_id(), instance);
    let missing = crate::history::data::CommitId(999);
    assert_eq!(
        settlement
            .repair_pending_publication_settlement(missing)
            .unwrap_err(),
        crate::publication::data::PublicationSettlementRepairError::RecoveryUnavailable {
            commit_id: missing
        }
    );
    drop(settlement);
    let (_, recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(execution, &native)
        .unwrap();
    assert!(recovered.codec_transition_pending());
}

#[test]
fn empty_destination_retained_native_basis_refuses_then_drop_allows_funded_retry() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let source = world(execution, true);
    let native = capture(&source.runtime);
    let (mut target, _sealed_receipt) = installed(2);
    let selected = basis(&target);
    assert!(selected.reference().target().is_empty());
    assert_eq!(target.history().immutable_commit_count(), 0);
    assert_eq!(target.visibility.active_snapshot_count(), 0);
    assert_destination_refused_unchanged(execution, &mut target, &native);
    assert!(selected.is_current());
    assert!(selected.reference().target().is_empty());
    assert_eq!(
        selected.inner.root.schema_authority().registry(),
        &target.config().schema.registry
    );
    drop(selected);
    let (_, recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(execution, &native)
        .unwrap();
    assert!(recovered.codec_transition_pending());
}

#[test]
fn published_destination_keeps_raw_source_exact_basis_and_usable_snapshot_after_refusal() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let source = world(execution, true);
    let native = capture(&source.runtime);
    let (mut target, _sealed_receipt) = installed(2);
    let authored = create(
        execution,
        &target,
        1,
        "destination authored source",
        SOURCE_VALUE,
        None,
    );
    let selected = basis(&target);
    let descriptor = selected.descriptor().clone();
    let snapshot = target
        .snapshots()
        .snapshot_for_observation(&selected.observation())
        .unwrap();
    let before = target
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    let raw = protected_payload(&target, authored);
    assert_destination_refused_unchanged(execution, &mut target, &native);
    assert!(selected.is_current());
    assert_eq!(basis(&target).descriptor(), &descriptor);
    assert_eq!(protected_payload(&target, authored), raw);
    assert_eq!(
        target
            .read_truth()
            .read_snapshot(&snapshot, execution)
            .unwrap()
            .unwrap(),
        before
    );
    assert_eq!(
        target
            .read_truth()
            .exact_snapshot_live_entity_kind(&snapshot, authored),
        Some(crate::identity::data::KindId(1))
    );
    target.snapshots().release_snapshot(&snapshot).unwrap();
}
