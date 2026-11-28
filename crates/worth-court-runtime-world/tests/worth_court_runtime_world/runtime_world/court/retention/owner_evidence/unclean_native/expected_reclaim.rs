use super::*;
use worth_relational::facade::mvcc::RelationalTransactionIntent;

#[test]
fn cold_reopen_restores_each_expected_share_until_its_original_is_released() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut fixture = Fixture::journal(&request);
    let expected = match fixture.offer(&request, "primary") {
        WorldDurableBootstrapOutcome::Performed(actual) => actual.product_branch().clone(),
        other => panic!("actual root: {other:?}"),
    };
    let predecessor = expected.selected_commit().ordinal();
    let cancellation = RuntimeWorldCancellationSource::new();
    for _ in 0..2 {
        let opened = fixture
            .world
            .publication_port()
            .open_publication_without_signal(
                &request,
                expected.clone(),
                fixture.recorder.program_source(&expected),
                RelationalTransactionIntent::ordinary(),
                &cancellation.token(),
                None,
            );
        assert!(matches!(opened, WorldOpenAttemptOutcome::Opened(_)));
    }
    drop(expected);
    let fixture = fixture.reopen(&request);
    let report = fixture
        .world
        .lifecycle_port()
        .close(&request, &fixture.world)
        .unwrap();
    assert_eq!(report.recovered_custody().len(), 2);
    fixture
        .world
        .retire_checkpoint_origin(&request, &fixture.native)
        .unwrap();
    drop(report);

    let fixture = fixture.reopen(&request);
    let page = fixture
        .world
        .inspection_port()
        .recovery_page(None, maximum())
        .unwrap();
    assert_eq!(page.rows().len(), 2);
    let recovery = fixture.world.recovery_port();
    for row in page.rows() {
        let effects = recovery.inspect_recovered_effects(row.handle()).unwrap();
        assert_eq!(
            effects.expected_head().selected_commit().ordinal(),
            predecessor
        );
        let released = recovery.release_recovered_effects(&request, effects);
        assert!(
            matches!(released, WorldRecoveredReleaseOutcome::Released(_)),
            "{released:?}"
        );
        assert!(recovery.inspect_recovered_effects(row.handle()).is_err());
    }
    drop((page, recovery));
    let fixture = fixture.reopen(&request);
    let page = fixture
        .world
        .inspection_port()
        .recovery_page(None, maximum())
        .unwrap();
    assert!(page.rows().is_empty());
    drop(page);
    fixture.finish(&request);
}
