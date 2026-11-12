//! Automatic membership admission uses the real Host service, not a manual release.
use super::*;
use crate::process::application::query_original_membership::MembershipJournal;
use worth_foundational::facade::{CanonicalDigestId, WorldOriginalFate};
use worth_query_execution::facade::program_support::QueryProgramSupportRecord;
use worth_query_host::facade::{
    application_installation::WorthQueryProgramSupportRetirementFailure,
    domain::WorthQueryProgramSupportRetirementDenial, WorthQueryApplicationHome,
};
use worth_runtime_world::facade::*;

type Home = WorthQueryApplicationHome<HomeIdentitySchema, HomeProgram>;
fn reopen(path: &std::path::Path, execution: ExecutionRequest<'_, '_>) -> Home {
    let request = fixture::request_with_initializer(
        path.to_path_buf(),
        build(),
        Arc::new(AtomicUsize::new(0)),
        |_, _, _| panic!("membership readmission must not initialize"),
    );
    with_bootstrap_advancement_from_request(execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|refused| panic!("actual membership opener: {refused:?}"))
}
fn counted(
    home: &Home,
    execution: &ExecutionRequest<'_, '_>,
    revision: CanonicalDigestId,
    expected: usize,
) {
    let actual = *home.runtime().installed_program().revision();
    assert_eq!(actual.as_bytes(), revision.bytes());
    let Err(WorthQueryProgramSupportRetirementFailure::Policy(
        WorthQueryProgramSupportRetirementDenial::CurrentBranches(inventory),
    )) = home.runtime().retire_program_support(execution, &actual)
    else {
        panic!("the genuine main branch still blocks retirement");
    };
    assert_eq!(inventory.current_branches(), 1);
    assert_eq!(inventory.retained_interpretations(), expected);
    assert_eq!(inventory.mandatory_custody(), 0);
}

#[test]
fn home_custody_keeps_published_original_membership_after_graceful_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    fresh_closed(directory.path(), execution);
    let home = reopen(directory.path(), execution);
    assert!(matches!(
        set(home.runtime(), 8),
        WorthQueryApplicationMutationOutcome::Committed { result: 8, .. }
    ));
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("actual source close: {refused:?}"))
        .reclaim();
    let revision = with_bootstrap_advancement_from_request(execution, |phase| {
        let (query, home, witness) = existing(directory.path(), &phase);
        let history = home
            .world()
            .inspection_port()
            .observe_program_activation_history(&execution)
            .unwrap();
        let originals = history.original_states().collect::<Vec<_>>();
        let [original] = originals.as_slice() else {
            panic!("one real admitted Query original");
        };
        assert_eq!(original.fate(), WorldOriginalFate::Published);
        let revision = original.program_revision();
        let mut journal = MembershipJournal::read_home(&home, &execution);
        // Fixture DATA exercises the cold consumer; no original-meaning token is fabricated.
        journal.acquire(original.attempt(), revision, &execution);
        assert_eq!(journal.ledger.counts(&revision), (1, 0));
        drop(originals);
        drop((journal, history, query, witness));
        close_world(home, &execution);
        revision
    })
    .unwrap();
    for _ in 0..2 {
        let home = reopen(directory.path(), execution);
        assert_eq!(balance(home.runtime()), 8);
        counted(&home, &execution, revision, 1);
        home.close(&execution)
            .unwrap_or_else(|refused| panic!("counted close: {refused:?}"))
            .reclaim();
    }
}

#[test]
fn home_custody_disposes_released_original_membership_before_service_after_graceful_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    fresh_closed(directory.path(), execution);
    let (identity, revision, attempt) =
        with_bootstrap_advancement_from_request(execution, |phase| {
            let (query, mut home, witness) = existing(directory.path(), &phase);
            let expected = home.select_query_root(&execution).unwrap();
            let cancellation = RuntimeWorldCancellationSource::new();
            let source_revision = *home.binding().selected().revision().bytes();
            let program_source = home.branch_program_source_owner().admit_source(
                source_revision,
                world_branch_program_source_commitment(&expected, source_revision).unwrap(),
            );
            let port = home.world().publication_port();
            let mut outcome = port.open_publication_without_signal(
                &execution,
                expected.clone(),
                program_source,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                &cancellation.token(),
                None,
            );
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            let opened = loop {
                match outcome {
                    WorldOpenAttemptOutcome::Opened(opened) => break opened,
                    WorldOpenAttemptOutcome::Pending(pending) => {
                        assert!(std::time::Instant::now() < deadline);
                        outcome = pending.advance(&execution);
                    }
                    other => panic!("actual acknowledged open: {other:?}"),
                }
            };
            let history = home
                .world()
                .inspection_port()
                .observe_program_activation_history(&execution)
                .unwrap();
            let originals = history.original_states().collect::<Vec<_>>();
            let [original] = originals.as_slice() else {
                panic!("one actual no-effect original");
            };
            assert_eq!(original.fate(), WorldOriginalFate::Open);
            assert_eq!(opened.program_revision(), original.program_revision());
            assert_eq!(opened.association(), original.association());
            let (attempt, revision) = (original.attempt(), original.program_revision());
            let mut journal = MembershipJournal::read_home(&home, &execution);
            let identity = journal.acquire(attempt, revision, &execution);
            assert_eq!(journal.ledger.counts(&revision), (1, 0));
            // Current warm release requires reopen. This establishes the genuine
            // Released precondition; the graceful consumer boundary starts below.
            drop(originals);
            drop((opened, journal, history, expected, query, witness, home));
            (identity, revision, attempt)
        })
        .unwrap();
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query, home, witness) = existing(directory.path(), &phase);
        let page = home
            .world()
            .inspection_port()
            .recovery_page(None, std::num::NonZeroUsize::MIN)
            .unwrap();
        let [row] = page.rows() else {
            panic!("one actual recoverable original");
        };
        let recovery = home.world().recovery_port();
        let claim = recovery.inspect_recovered_effects(row.handle()).unwrap();
        assert!(claim.native_receipt().is_none());
        let mut outcome = recovery.release_recovered_effects(&execution, claim);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            match outcome {
                WorldRecoveredReleaseOutcome::Released(released) => {
                    drop(released);
                    break;
                }
                WorldRecoveredReleaseOutcome::Pending(claim) => {
                    assert!(std::time::Instant::now() < deadline);
                    outcome = recovery.release_recovered_effects(&execution, claim);
                }
                other => panic!("actual no-effect disposal: {other:?}"),
            }
        }
        let history = home
            .world()
            .inspection_port()
            .observe_program_activation_history(&execution)
            .unwrap();
        assert_eq!(
            history.original_state(attempt).unwrap().fate(),
            WorldOriginalFate::Released
        );
        let journal = MembershipJournal::read_home(&home, &execution);
        assert_eq!(journal.ledger.counts(&revision), (1, 0));
        drop((journal, history, page, recovery, query, witness));
        close_world(home, &execution);
    })
    .unwrap();
    let home = reopen(directory.path(), execution);
    counted(&home, &execution, revision, 0);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("disposed close: {refused:?}"))
        .reclaim();
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query, home, witness) = existing(directory.path(), &phase);
        let journal = MembershipJournal::read_home(&home, &execution);
        assert_eq!(journal.ledger.counts(&revision), (0, 0));
        assert!(journal.ledger.obligations().next().is_none());
        let frame = journal.read_frame(3, &execution);
        assert!(matches!(frame.record(),
            QueryProgramSupportRecord::Release { identity: actual } if *actual == identity));
        assert_eq!(
            journal.ledger.position(),
            5,
            "one funded release then the retirement inspection's active abort"
        );
        drop((journal, query, witness));
        close_world(home, &execution);
    })
    .unwrap();
}
