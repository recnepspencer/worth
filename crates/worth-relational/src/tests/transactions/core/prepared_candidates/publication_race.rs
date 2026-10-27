use super::*;

#[test]
fn publication_port_performs_one_exact_candidate_and_reports_the_loser_stale() {
    let host_request = std::sync::Arc::new(crate::test_execution_policy::serial_request());
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "publication-race-anchor");
    let anchor_commit_id = anchor.commit.commit_id;
    let commit_count_before = runtime.history().immutable_commit_count();
    let expected = crate::tests::support::test_owner_main_basis(&runtime)
        .expect("main basis is admitted")
        .descriptor()
        .clone();
    let visible_entity_count_before = runtime
        .read_truth()
        .read_observation(
            execution,
            &crate::tests::support::test_owner_main_basis(&runtime)
                .expect("counting basis")
                .observation(),
        )
        .expect("read request remains live")
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
        .prepare_branch_transaction(execution, first)
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
        .prepare_branch_transaction(execution, second)
        .expect("second candidate prepares");
    let start = Arc::new(Barrier::new(3));
    let (first_done, first_completion) = std::sync::mpsc::sync_channel(1);
    let first_start = Arc::clone(&start);
    let first_port = runtime.publication_port();
    let first_thread = std::thread::spawn({
        let publication_request = std::sync::Arc::clone(&host_request);
        move || {
            let execution = &worth_execution::ExecutionRequest::serial(&publication_request);

            first_start.wait();
            let outcome = first_port.compare_and_publish(execution, first);
            first_done
                .send(())
                .expect("first completion receiver lives");
            outcome
        }
    });
    let (second_done, second_completion) = std::sync::mpsc::sync_channel(1);
    let second_start = Arc::clone(&start);
    let second_port = runtime.publication_port();
    let second_thread = std::thread::spawn({
        let publication_request = std::sync::Arc::clone(&host_request);
        move || {
            let execution = &worth_execution::ExecutionRequest::serial(&publication_request);

            second_start.wait();
            let outcome = second_port.compare_and_publish(execution, second);
            second_done
                .send(())
                .expect("second completion receiver lives");
            outcome
        }
    });
    start.wait();
    let first_thread_result = join_completed_worker(
        first_completion.recv_timeout(std::time::Duration::from_secs(1)),
        first_thread,
        std::time::Duration::from_secs(1),
        "first same-reference publisher completes within one second",
    );
    let second_thread_result = join_completed_worker(
        second_completion.recv_timeout(std::time::Duration::from_secs(1)),
        second_thread,
        std::time::Duration::from_secs(1),
        "second same-reference publisher completes within one second",
    );
    let first_outcome = first_thread_result;
    let second_outcome = second_thread_result;
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
        .read_observation(execution, &observation)
        .expect("read request remains live")
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
            .canonical_commit_envelope(execution, performed.canonical_commit().commit.commit_id)
            .unwrap()
            .map(|envelope| envelope.as_ref().clone()),
        Some(performed.canonical_commit().clone()),
        "the ordinary replay surface resolves the root-owned artifact"
    );
    assert_eq!(
        runtime
            .replay()
            .canonical_commit_envelope(execution, performed.canonical_commit().commit.commit_id)
            .unwrap()
            .map(|envelope| envelope.as_ref().clone()),
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
        .read_patch_stream(execution, PatchStreamRequest::default())
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
