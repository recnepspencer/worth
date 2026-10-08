use crate::facade::identity::{KindId, PartitionId};
use crate::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};
use crate::tests::support::*;
use std::sync::{Arc, Barrier};

#[test]
fn preparation_is_truth_effect_free_and_discard_releases_reservations() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "prepared-anchor");

    let branch_cells_before = runtime.history.branch_cells_snapshot();
    let envelopes_before = runtime.history().commit_envelopes_snapshot();
    let commit_count_before = runtime.history().immutable_commit_count();
    let patch_count_before = runtime.history.recorded_patch_position_count();
    let bundle_before = runtime.publication().latest_bundle();
    let diagnostics_before = runtime.publication().diagnostic_access().artifact_count();
    let durable_count_before = runtime.durability().durable_log().len();
    let candidate_cost_before = runtime
        .history
        .sharing_costs_for_branch(&BranchId("main".to_owned()));

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("prepared-discard"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("all fallible preparation succeeds without publication");

    assert_eq!(runtime.history.branch_cells_snapshot(), branch_cells_before);
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        envelopes_before
    );
    assert_eq!(
        runtime.history().immutable_commit_count(),
        commit_count_before
    );
    assert_eq!(
        runtime.history.recorded_patch_position_count(),
        patch_count_before
    );
    assert_eq!(runtime.publication().latest_bundle(), bundle_before);
    assert_eq!(
        runtime.publication().diagnostic_access().artifact_count(),
        diagnostics_before
    );
    assert_eq!(
        runtime.durability().durable_log().len(),
        durable_count_before
    );
    assert_eq!(candidate.reservation_count(), 1);

    let discarded = runtime
        .discard_prepared_candidate(candidate)
        .expect("the owner consumes the candidate through explicit discard");
    assert_eq!(discarded.released_record_reservation_count(), 1);
    assert_eq!(discarded.branch(), &BranchId("main".to_owned()));
    assert_eq!(runtime.history.branch_cells_snapshot(), branch_cells_before);
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        envelopes_before
    );
    assert_eq!(
        runtime.history().immutable_commit_count(),
        commit_count_before
    );
    assert_eq!(
        runtime.history.recorded_patch_position_count(),
        patch_count_before
    );
    assert_eq!(runtime.publication().latest_bundle(), bundle_before);
    assert_eq!(
        runtime.publication().diagnostic_access().artifact_count(),
        diagnostics_before
    );
    assert_eq!(
        runtime.durability().durable_log().len(),
        durable_count_before
    );
    let candidate_cost_after = runtime
        .history
        .sharing_costs_for_branch(&BranchId("main".to_owned()));
    assert_eq!(
        candidate_cost_after.candidate_preparations,
        candidate_cost_before.candidate_preparations + 1
    );
    assert_eq!(
        candidate_cost_after.candidate_discards,
        candidate_cost_before.candidate_discards + 1
    );
    assert_eq!(
        candidate_cost_after.publication_attempts,
        candidate_cost_before.publication_attempts
    );
}

#[test]
fn prepared_root_materializes_exactly_the_declared_write_partitions() {
    let runtime = runtime_with_test_schema();
    create_entity_in_partition(&runtime, "main-anchor", PartitionId::main());
    create_entity_in_partition(&runtime, "second-anchor", PartitionId(29));
    create_entity_in_partition(&runtime, "untouched-anchor", PartitionId(41));

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            create_batch("main-write", PartitionId::main()),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            create_batch("second-write", PartitionId(29)),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let declared_write_partition_count = transaction.footprint().write_partitions().len() as u64;
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("candidate root construction succeeds");
    let (touched, reused) = candidate.materialization_counts();

    assert_eq!(touched, declared_write_partition_count);
    assert_eq!(touched, 2);
    assert_eq!(reused, 1, "the undeclared third partition must be reused");
    runtime
        .discard_prepared_candidate(candidate)
        .expect("materialization candidate remains discardable");
}

#[test]
fn prepared_candidate_is_sendable_across_one_worker_boundary() {
    fn assert_send<T: Send>() {}
    fn assert_clone_send_sync<T: Clone + Send + Sync>() {}
    assert_send::<crate::facade::mvcc::PreparedRelationalCommitCandidate>();
    assert_clone_send_sync::<crate::facade::mvcc::RelationalPublicationPort>();
}

#[test]
fn publication_port_performs_one_exact_candidate_and_reports_the_loser_stale() {
    let runtime = runtime_with_test_schema();
    let anchor = create_entity_outcome(&runtime, "publication-race-anchor");
    let anchor_commit_id = anchor.commit.commit_id;
    let commit_count_before = runtime.history().immutable_commit_count();
    let expected = crate::tests::support::test_owner_main_basis(&runtime)
        .expect("main basis is admitted")
        .descriptor()
        .clone();
    let visible_entity_count_before = runtime
        .read_truth()
        .read_observation(
            &crate::tests::support::test_owner_main_basis(&runtime)
                .expect("counting basis")
                .observation(),
        )
        .expect("counting observation reads")
        .entities()
        .len();

    let mut first = runtime
        .begin_branch_transaction(
            &crate::tests::support::test_owner_main_basis(&runtime).expect("first basis"),
            crate::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("first transaction binds");
    first
        .push_batch(
            batch_create("first-race-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let first = runtime
        .prepare_branch_transaction(
            first,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("first candidate prepares");

    let mut second = runtime
        .begin_branch_transaction(
            &crate::tests::support::test_owner_main_basis(&runtime).expect("second basis"),
            crate::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("second transaction binds");
    second
        .push_batch(
            batch_create("second-race-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let second = runtime
        .prepare_branch_transaction(
            second,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("second candidate prepares");
    let start = Arc::new(Barrier::new(3));
    let (first_done, first_completion) = std::sync::mpsc::sync_channel(1);
    let first_start = Arc::clone(&start);
    let first_port = runtime.publication_port();
    let first_thread = std::thread::spawn(move || {
        first_start.wait();
        let outcome = first_port.compare_and_publish(first);
        first_done
            .send(())
            .expect("first completion receiver lives");
        outcome
    });
    let (second_done, second_completion) = std::sync::mpsc::sync_channel(1);
    let second_start = Arc::clone(&start);
    let second_port = runtime.publication_port();
    let second_thread = std::thread::spawn(move || {
        second_start.wait();
        let outcome = second_port.compare_and_publish(second);
        second_done
            .send(())
            .expect("second completion receiver lives");
        outcome
    });
    start.wait();
    first_completion
        .recv_timeout(std::time::Duration::from_secs(1))
        .expect("first same-reference publisher completes within one second");
    second_completion
        .recv_timeout(std::time::Duration::from_secs(1))
        .expect("second same-reference publisher completes within one second");
    let first_outcome = first_thread.join().expect("first publisher joins");
    let second_outcome = second_thread.join().expect("second publisher joins");
    let (performed, stale) = match (first_outcome, second_outcome) {
        (
            crate::mvcc::RelationalPublicationOutcome::Performed(performed),
            crate::mvcc::RelationalPublicationOutcome::Stale(stale),
        )
        | (
            crate::mvcc::RelationalPublicationOutcome::Stale(stale),
            crate::mvcc::RelationalPublicationOutcome::Performed(performed),
        ) => (performed, stale),
        outcomes => {
            panic!("the same-reference race must have one winner and one stale loser: {outcomes:?}")
        }
    };
    assert_eq!(
        performed.next_basis().descriptor().branch_id(),
        expected.branch_id()
    );
    assert_ne!(performed.next_basis().descriptor(), &expected);
    assert_eq!(
        performed
            .canonical_commit()
            .patch
            .authoritative_record_patches
            .len(),
        1
    );
    assert_eq!(
        performed.projection_posture(),
        crate::mvcc::RelationalPublicationProjectionPosture::CanonicalRootCurrentOptionalProjectionsDeferred
    );
    assert_eq!(
        performed.durability_posture(),
        crate::mvcc::RelationalPublicationDurabilityPosture::OwnerAcknowledgementDeferred
    );
    assert!(performed
        .next_basis()
        .inner
        .root
        .is_complete(&runtime.services.symbols.interner_snapshot()));
    let observation = performed.next_basis().observation();
    let created_entity = match &performed
        .canonical_commit()
        .patch
        .authoritative_record_patches[0]
        .target
    {
        crate::transactions::data::RecordRef::Entity(entity_id) => *entity_id,
        crate::transactions::data::RecordRef::Relation(_) => {
            panic!("the candidate creates one entity")
        }
    };
    let visible = runtime
        .read_truth()
        .read_observation(&observation)
        .expect("the performed basis reads its selected root");
    assert!(visible.get_entity(created_entity).is_some());
    assert_eq!(
        visible.entities().len(),
        visible_entity_count_before + 1,
        "the selected root contains the winner only, never a merged loser write"
    );
    assert_eq!(
        observation.commit_receipt(),
        Some(&performed.canonical_commit().commit)
    );
    assert_eq!(
        observation.canonical_patch(),
        Some(&performed.canonical_commit().patch)
    );
    assert_eq!(
        observation.correctness_index(),
        Some(crate::branch::RelationalRootCorrectnessIndex::AuthoritativeFallback)
    );
    assert_eq!(
        runtime
            .history()
            .immutable_commit_receipt(performed.canonical_commit().commit.commit_id),
        Some(performed.canonical_commit().commit.clone())
    );
    assert_eq!(
        runtime
            .replay()
            .canonical_commit_envelope(performed.canonical_commit().commit.commit_id),
        Some(performed.canonical_commit().clone()),
        "the ordinary replay surface resolves the root-owned artifact"
    );
    assert_eq!(
        runtime
            .replay()
            .canonical_commit_envelope_owned(performed.canonical_commit().commit.commit_id),
        Some(performed.canonical_commit().clone()),
        "replay input resolves the same canonical root-owned artifact"
    );
    assert_eq!(
        runtime.history().immutable_commit_count(),
        commit_count_before + 1
    );
    let ancestry = runtime
        .history()
        .ancestor_closure_by_commit_id_order(performed.canonical_commit().commit.commit_id);
    assert!(ancestry.contains(&anchor_commit_id));
    assert!(ancestry.contains(&performed.canonical_commit().commit.commit_id));
    assert!(runtime
        .history()
        .recent_commit_ids(Some(&BranchId("main".to_owned())), 8)
        .contains(&performed.canonical_commit().commit.commit_id));
    let stream = runtime
        .publication()
        .read_patch_stream(PatchStreamRequest::default())
        .expect("the performed root's patch is immediately stream-visible");
    let expected_patch =
        crate::publication::patch::data::PublishedAuthoritativePatchEnvelope::from_canonical(
            performed.patch_position(),
            &performed.canonical_commit().patch,
        );
    assert!(stream.patches.iter().any(|patch| patch == &expected_patch));
    assert_eq!(
        stream.latest_commit_id,
        Some(performed.canonical_commit().commit.commit_id)
    );

    assert_eq!(stale.expected(), &expected);
    assert_eq!(stale.observed(), performed.next_basis().descriptor());
    assert_eq!(
        runtime.history().immutable_commit_count(),
        commit_count_before + 1,
        "the losing candidate must not enter canonical history"
    );
    assert_eq!(
        runtime.history.pending_canonical_publication_route_count(),
        0,
        "the stale candidate releases its private canonical route reservation"
    );
}

fn create_batch(name: &str, partition_id: PartitionId) -> WorkerIntentBatch {
    WorkerIntentBatch::new(format!("batch-{name}")).push(MutationIntent::Create(
        CreateIntent::Entity(crate::transactions::data::EntitySpec {
            partition_id,
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw(name),
            fields: entity_fields_for_runtime_name(name),
        }),
    ))
}

fn entity_fields_for_runtime_name(name: &str) -> crate::transactions::data::AspectFieldPatch {
    single_string_aspect_field_patch(aspect_key("name"), field_key("name"), name)
}
