//! Actual validated proposal bindings retain their allocation and occurrence owners.
use crate::identity::data::EntityId;
use crate::mvcc::ValidatedRelationalProposal;
use crate::tests::support::*;
use crate::transactions::data::CreatedEntityRef;
use worth_execution::ExecutionRequest;

#[test]
fn validated_sibling_proposals_keep_their_observed_identity_through_reverse_commit() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let mut runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "custody-basis");
    fork(&mut runtime, "custody-a");
    fork(&mut runtime, "custody-b");

    let a = propose(execution, &runtime, "custody-a", "a-record");
    let a_id = proposed_record(&a, "a-record");
    let a_occurrence = a.proposal_identity.commit_id();
    let a_version = a.proposal_identity.proposed_version_id();
    let b = propose(execution, &runtime, "custody-b", "b-record");
    let b_id = proposed_record(&b, "b-record");
    let b_occurrence = b.proposal_identity.commit_id();
    assert_ne!(a_id, b_id, "live proposals own different real slots");
    assert!(a_occurrence < b_occurrence);
    assert_eq!(runtime.record_identity.pending_snapshot().len(), 2);

    let b = publish(execution, &runtime, b);
    assert_eq!(b.created_entity(&reference("b-record")), Some(b_id));
    assert_eq!(b.commit.commit_id, b_occurrence);
    let a = publish(execution, &runtime, a);
    assert_eq!(a.created_entity(&reference("a-record")), Some(a_id));
    assert_eq!(a.commit.commit_id, a_occurrence);
    assert_eq!(a.version_id, a_version);
    assert!(b.patch_position() < a.patch_position());
    assert!(runtime.record_identity.pending_snapshot().is_empty());
    for (outcome, id, name) in [(&a, a_id, "a-record"), (&b, b_id, "b-record")] {
        let records = runtime
            .read_truth()
            .project_historical_version(outcome.version_id)
            .all_authoritative_entity_records(execution)
            .unwrap();
        let record = records
            .iter()
            .find(|record| record.entity_id == id)
            .unwrap();
        assert_eq!(read_entity_name(record).as_deref(), Some(name));
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn discarded_and_stale_validated_proposals_release_custody_and_burn_append_gaps() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let before = runtime.history().immutable_commit_count();
    let discarded = propose(execution, &runtime, "main", "discarded-record");
    let discarded_id = proposed_record(&discarded, "discarded-record");
    let discarded_occurrence = discarded.proposal_identity.commit_id();
    assert_eq!(runtime.record_identity.pending_snapshot().len(), 1);
    assert_eq!(runtime.history().immutable_commit_count(), before);
    drop(discarded);
    assert!(runtime.record_identity.pending_snapshot().is_empty());

    let stale = propose(execution, &runtime, "main", "stale-record");
    let stale_id = proposed_record(&stale, "stale-record");
    let stale_occurrence = stale.proposal_identity.commit_id();
    assert_ne!(stale_id, discarded_id);
    assert!(stale_occurrence > discarded_occurrence);
    let winner = create_entity_outcome(execution, &runtime, "winner-record");
    assert!(winner.commit.commit_id > stale_occurrence);
    let denied = runtime
        .prepare_validated_proposal(execution, stale)
        .unwrap_err();
    assert!(matches!(
        denied,
        TransactionCommitError::Conflict {
            error: crate::transactions::data::CommitConflict {
                class: crate::transactions::data::ConflictClass::StaleValidationBasis { .. },
                ..
            },
            ..
        }
    ));
    assert!(runtime.record_identity.pending_snapshot().is_empty());
    let continued = create_entity_outcome(execution, &runtime, "continued-record");
    let continued_id = continued
        .created_entity(&reference("continued-record"))
        .unwrap();
    assert!(continued_id.slot_index() > stale_id.slot_index());
    assert_eq!(runtime.history().immutable_commit_count(), before + 2);
    for outcome in [&winner, &continued] {
        let records = runtime
            .read_truth()
            .project_historical_version(outcome.version_id)
            .all_authoritative_entity_records(execution)
            .unwrap();
        assert!(!records
            .iter()
            .any(|record| [discarded_id, stale_id].contains(&record.entity_id)));
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn runtime_fork_releases_only_its_copied_pending_entries_while_source_keeps_custody() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "fork-custody-basis");
    let proposal = propose(execution, &runtime, "main", "source-only-record");
    let source_id = proposed_record(&proposal, "source-only-record");
    let pending = runtime.record_identity.pending_snapshot();
    assert_eq!(pending.len(), 1);
    let detached = runtime.fork().unwrap();
    assert!(detached.record_identity.pending_snapshot().is_empty());
    assert_eq!(runtime.record_identity.pending_snapshot(), pending);
    let detached_commit = create_entity_outcome(execution, &detached, "detached-only-record");
    let detached_id = detached_commit
        .created_entity(&reference("detached-only-record"))
        .unwrap();
    assert!(
        detached_id.slot_index() > source_id.slot_index(),
        "copied append gap stays burned"
    );
    assert_eq!(runtime.record_identity.pending_snapshot(), pending);
    let source = publish(execution, &runtime, proposal);
    assert_eq!(
        source.created_entity(&reference("source-only-record")),
        Some(source_id)
    );
    assert!(runtime.record_identity.pending_snapshot().is_empty());
    let detached_records = detached
        .read_truth()
        .project_historical_version(detached_commit.version_id)
        .all_authoritative_entity_records(execution)
        .unwrap();
    assert!(!detached_records
        .iter()
        .any(|record| record.entity_id == source_id));
    release_test_commit_snapshot(&runtime, &source);
    release_test_commit_snapshot(&detached, &detached_commit);
}

fn reference(key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw(key),
    }
}
fn propose(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &str,
    key: &str,
) -> ValidatedRelationalProposal {
    let mut transaction = test_owner_begin_transaction_for_branch(runtime, BranchId(branch.into()));
    transaction
        .push_batch(batch_create(key), execution.allocation_policy())
        .unwrap();
    transaction.validate(execution, runtime).unwrap()
}
fn proposed_record(proposal: &ValidatedRelationalProposal, key: &str) -> EntityId {
    let id = proposal
        .proposed_invariant_state
        .created_entities()
        .resolve(&reference(key))
        .unwrap();
    let view = crate::validation::engine::state_view::InvariantStateView::new(
        proposal.proposed_invariant_state.working_state(),
        proposal.proposal_identity.proposed_version_id(),
    );
    assert!(
        view.entity_metadata(id).is_some(),
        "binding names actual proposed truth"
    );
    id
}
fn publish(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    proposal: ValidatedRelationalProposal,
) -> CommitResult {
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("validated owner publication must perform")
    };
    runtime.settle_performed_publication(performed).unwrap()
}
fn fork(runtime: &mut RelationalRuntime, name: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime.fork_branch(BranchId(name.into()), source).unwrap();
}
