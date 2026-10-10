use crate::tests::support::*;

#[test]
fn expired_candidate_is_typed_deferred_before_reference_movement() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .publication(crate::facade::config::PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 1,
            max_active_snapshot_handles: 8,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 1,
            candidate_max_lifetime_millis: 0,
        })
        .build();
    let before = crate::tests::support::test_owner_main_basis(&runtime).unwrap();
    let cost_scope = crate::facade::inspection::RelationalMvccCostScope::capture(
        &runtime,
        vec![runtime.main_branch_identity()],
    );
    let mut transaction = runtime
        .begin_branch_transaction(
            &before,
            crate::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            batch_create("expired-candidate"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    assert_eq!(runtime.reap_expired_prepared_candidates(), 1);
    assert!(matches!(
        runtime.publication_port().compare_and_publish(candidate),
        crate::mvcc::RelationalPublicationOutcome::Deferred(
            crate::mvcc::RelationalPublicationDeferred::CandidateLifetimeExpired {
                maximum_lifetime_millis: 0,
            }
        )
    ));
    assert_eq!(
        crate::tests::support::test_owner_main_basis(&runtime)
            .unwrap()
            .descriptor(),
        before.descriptor()
    );
    assert_eq!(
        runtime.history.pending_canonical_publication_route_count(),
        0
    );
    let reaped_cost = runtime.observe_mvcc_counters(&cost_scope).unwrap();
    assert_eq!(reaped_cost.retention_cost_delta().candidate_acquires, 1);
    assert_eq!(reaped_cost.retention_cost_delta().candidate_releases, 1);

    let replacement = prepared_write(&runtime, &before, "post-reap-replacement").unwrap();
    runtime.discard_prepared_candidate(replacement).unwrap();
    let terminal_cost = runtime.observe_mvcc_counters(&cost_scope).unwrap();
    assert_eq!(terminal_cost.retention_cost_delta().candidate_acquires, 2);
    assert_eq!(terminal_cost.retention_cost_delta().candidate_releases, 2);
}

#[test]
fn candidate_that_expires_while_waiting_for_coordination_does_not_move_reference() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .publication(crate::facade::config::PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 8,
            max_active_snapshot_handles: 8,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 1,
            candidate_max_lifetime_millis: 1_000,
        })
        .build();
    let basis = crate::tests::support::test_owner_main_basis(&runtime).unwrap();
    let candidate = prepared_write(&runtime, &basis, "waited-expiry").unwrap();
    let publication_cell = candidate.publication_cell_for_test();
    let coordination = std::sync::Arc::clone(publication_cell.coordination());
    let held_coordination = coordination.enter();
    let before = crate::tests::support::test_owner_main_basis(&runtime).unwrap();
    let port = runtime.publication_port();

    let publisher = std::thread::spawn(move || port.compare_and_publish(candidate));
    let wait_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while coordination.wait_count() == 0 && std::time::Instant::now() < wait_deadline {
        std::thread::yield_now();
    }
    assert_eq!(coordination.wait_count(), 1);
    std::thread::sleep(std::time::Duration::from_millis(1_100));
    drop(held_coordination);

    assert!(matches!(
        publisher.join().unwrap(),
        crate::mvcc::RelationalPublicationOutcome::Deferred(
            crate::mvcc::RelationalPublicationDeferred::CandidateLifetimeExpired {
                maximum_lifetime_millis: 1_000,
            }
        )
    ));
    assert_eq!(
        crate::tests::support::test_owner_main_basis(&runtime)
            .unwrap()
            .descriptor(),
        before.descriptor()
    );
    assert_eq!(
        runtime.history.pending_canonical_publication_route_count(),
        0
    );
    let replacement = prepared_write(&runtime, &basis, "post-wait-expiry").unwrap();
    runtime.discard_prepared_candidate(replacement).unwrap();
}

#[test]
fn foreign_publication_port_denies_before_reference_movement() {
    let source = runtime_with_test_schema();
    create_entity(&source, "foreign-port-anchor");
    let before = crate::tests::support::test_owner_main_basis(&source).expect("source basis");
    let commit_count_before = source.history().immutable_commit_count();
    let mut transaction = source
        .begin_branch_transaction(
            &before,
            crate::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("source transaction binds");
    transaction
        .push_batch(
            batch_create("foreign-port-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = source
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("source candidate prepares");
    let foreign = runtime_with_test_schema();

    match foreign.publication_port().compare_and_publish(candidate) {
        crate::mvcc::RelationalPublicationOutcome::Denied(
            crate::mvcc::RelationalPublicationDenial::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            },
        ) => {
            assert_eq!(expected_runtime_instance_id, foreign.runtime_instance_id());
            assert_eq!(actual_runtime_instance_id, source.runtime_instance_id());
        }
        outcome => panic!("foreign port must return its typed denial: {outcome:?}"),
    }
    assert_eq!(
        crate::tests::support::test_owner_main_basis(&source)
            .expect("source basis remains current")
            .descriptor(),
        before.descriptor()
    );
    assert_eq!(
        source.history().immutable_commit_count(),
        commit_count_before
    );
}

#[test]
fn candidate_population_exhaustion_is_typed_and_released_by_discard() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .publication(crate::facade::config::PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 8,
            max_active_snapshot_handles: 8,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 1,
            candidate_max_lifetime_millis: 30_000,
        })
        .build();
    let basis = crate::tests::support::test_owner_main_basis(&runtime).unwrap();
    let first = prepared_write(&runtime, &basis, "candidate-one").unwrap();

    assert!(matches!(
        prepared_write(&runtime, &basis, "candidate-two"),
        Err(
            crate::transactions::data::TransactionCommitError::PublicationDeferred {
                deferred: crate::mvcc::RelationalPublicationDeferred::CandidateCapacityExhausted {
                    maximum_candidates: 1,
                },
                ..
            }
        )
    ));
    runtime.discard_prepared_candidate(first).unwrap();
    let replacement = prepared_write(&runtime, &basis, "candidate-replacement").unwrap();
    runtime.discard_prepared_candidate(replacement).unwrap();
}

fn prepared_write(
    runtime: &crate::runtime::RelationalRuntime,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
    key: &str,
) -> Result<
    crate::mvcc::PreparedRelationalCommitCandidate,
    crate::transactions::data::TransactionCommitError,
> {
    let mut transaction = runtime
        .begin_branch_transaction(basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch_create(key),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    runtime.prepare_branch_transaction(
        transaction,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
}
