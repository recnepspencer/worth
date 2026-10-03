use super::*;

/// A companion whose reservation is always held by another publisher.
#[derive(Debug)]
struct HeldReservationCompanion;

impl RelationalPublicationCompanion for HeldReservationCompanion {
    fn prepare(
        &self,
        _context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        Err(CompanionPreflightStop::TopologyPending)
    }
}

fn register(
    runtime: &RelationalRuntime,
    participant: Arc<dyn RelationalPublicationCompanion>,
) -> impl Sized {
    runtime
        .publication_companion_port()
        .begin_required_registration()
        .expect("required registration begins")
        .activate(
            participant,
            CompanionPreflightBudget {
                maximum_work_visits: 16,
                maximum_preparation_bytes: 4_096,
            },
        )
        .expect("the participant is active")
}

fn prepare_on_main(
    runtime: &RelationalRuntime,
    key: &str,
) -> crate::mvcc::PreparedRelationalCommitCandidate {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(batch_create(key))
        .expect("candidate stages");
    runtime
        .prepare_branch_transaction(transaction)
        .expect("candidate prepares")
}

#[test]
fn companion_root_mismatch_after_a_same_head_winner_reports_the_stale_head() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "same-head-anchor");
    let handle = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let selected = runtime
        .read_truth()
        .positioned_snapshot(&handle)
        .expect("owner snapshot has its canonical position");
    let pending = runtime
        .publication_companion_port()
        .begin_required_registration()
        .expect("required registration begins");
    let cell = pending
        .mint_branch_cell(&selected, Arc::new(0_u64))
        .expect("the cell belongs to the selected source");
    let calls = Arc::new(AtomicUsize::new(0));
    let _registration = pending
        .activate(
            Arc::new(CountingCompanion {
                cell: cell.clone(),
                calls: Arc::clone(&calls),
            }),
            CompanionPreflightBudget {
                maximum_work_visits: 16,
                maximum_preparation_bytes: 4_096,
            },
        )
        .expect("the participant is active");

    let winner = prepare_on_main(&runtime, "same-head-winner");
    let loser = prepare_on_main(&runtime, "same-head-loser");
    let RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(winner)
    else {
        panic!("the first same-head publisher performs");
    };
    let committed = runtime
        .settle_performed_publication(performed)
        .expect("the winner settles");
    // The companion now selects the winner's root, so its preflight reports
    // SelectedSourceMismatch; the moved head outranks that deferral.
    let RelationalPublicationOutcome::Stale(stale) =
        runtime.publication_port().compare_and_publish(loser)
    else {
        panic!("a same-head loser observes the stale head, not a companion deferral");
    };
    assert_ne!(stale.observed(), stale.expected());
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    release_test_commit_snapshot(&runtime, &committed);
    runtime
        .snapshots()
        .release_snapshot(&handle)
        .expect("the selected snapshot releases");
}

#[test]
fn companion_contention_on_an_unchanged_head_stays_a_retryable_deferral() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "unchanged-head-anchor");
    let _registration = register(&runtime, Arc::new(HeldReservationCompanion));
    let candidate = prepare_on_main(&runtime, "unchanged-head-candidate");
    assert!(matches!(
        runtime.publication_port().compare_and_publish(candidate),
        RelationalPublicationOutcome::Deferred(RelationalPublicationDeferred::CompanionPreflight(
            CompanionPreflightStop::TopologyPending
        ))
    ));
}
