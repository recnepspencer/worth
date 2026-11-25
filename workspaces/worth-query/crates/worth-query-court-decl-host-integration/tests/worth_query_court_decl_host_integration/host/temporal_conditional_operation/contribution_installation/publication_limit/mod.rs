pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::publication_limit::*;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::directory_home::{
    unique_directory, with_directory_bootstrap, DirectoryHomeGuard,
};

#[test]
fn explicit_publication_limit_survives_checkpoint_installation_and_enforces_commit_boundary() {
    let host_request =
        worth_query_testkit_host::facade::host::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let open = |dir: std::path::PathBuf, maximum: Option<usize>| {
        let mut app_limits = limits();
        if let Some(max) = maximum {
            app_limits = app_limits
                .with_maximum_publication_records(std::num::NonZeroUsize::new(max).unwrap());
        }
        let home = with_directory_bootstrap(|phase| {
            WorthQueryApplicationHomeRequest::new(
                WorthQueryApplicationHomeLocation::Directory(dir.clone()),
                validated_program(),
                application_installation::WorthQueryApplicationProgramRoster::new(),
                TemporalHostSchema::declaration().unwrap(),
                (configuration(),),
                ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
                app_limits,
                |_phase, graph, installed| {
                    let principal = installed
                        .principal_binding(TemporalPrincipalBinding::reference())
                        .unwrap();
                    seed_graph(graph, &principal, "original", 0, 1, true);
                    Ok(())
                },
            )
            .open_in_advancement(phase)
        })
        .expect("a record budget governs new commits, not valid Directory Home recovery");
        DirectoryHomeGuard::wrap(home, dir)
    };

    let dir = unique_directory("publication-limit");
    let source = open(dir, None);
    let original = read_intent(&source);
    let dir = source.checkpoint_and_close_for_reopen();

    // Reopen the SAME Directory Home with maximum_publication_records = 1.
    // The budget must refuse at PreparingNative with the exact Native cause.
    let application = open(dir.clone(), Some(1));
    assert_eq!(read_intent(&application), original);
    {
        let installed = application.conditional::<TemporalConditional>().unwrap();
        let installed = installed.as_ref().as_ref().unwrap();
        let outcome = change_input(
            request_execution,
            &application,
            &installed.invariant,
            application.current_world(),
            "changed",
        );
        let primary_graph::WorthQueryApplicationCommitOutcome::ProductPreparationPending(pending) =
            outcome
        else {
            panic!("expected PreparingNative refusal under publication record budget on Directory Home, got {outcome:?}");
        };
        assert_eq!(
            pending.stage(),
            primary_graph::WorthQueryProductPreparationStage::PreparingNative,
            "Native patch record budget must stop before any Native effect"
        );
        let Some(worth_relational::facade::mvcc::TransactionCommitError::Publication {
            error, ..
        }) = pending.native_preparation_error()
        else {
            panic!(
                "expected the exact Native publication cause on PreparingNative pending custody, got {:?}",
                pending.native_preparation_error()
            );
        };
        assert_eq!(error.detail, "patch record budget exceeded");
        let _recovery = pending
            .try_into_recovery()
            .expect("PreparingNative open hands off cleanly for reopen release");
        assert_eq!(
            read_intent(&application),
            original,
            "rejected publication leaves authoritative fields unchanged"
        );
    }
    // End the live open so the SAME Directory Home reopens and durably releases
    // the proven-no-effect open without resetting the directory.
    let (home, _path) = application.into_home();
    drop(home);

    let reopened = open(dir.clone(), Some(1));
    assert_source_change_key_absent(&reopened);
    assert_eq!(read_intent(&reopened), original);
    let dir = reopened.close_for_reopen();

    // Reopen the SAME recovered Directory Home (no directory reset) under the
    // sufficient budget and retry the EXACT SAME `"source-change"` key.
    let sufficient = open(dir.clone(), Some(65_536));
    assert_eq!(read_intent(&sufficient), original);
    {
        let installed = sufficient.conditional::<TemporalConditional>().unwrap();
        let installed = installed.as_ref().as_ref().unwrap();
        let outcome = change_input(
            request_execution,
            &sufficient,
            &installed.invariant,
            sufficient.current_world(),
            "changed",
        );
        outcome
            .require_committed()
            .expect("same real command key commits on the recovered Home once within allowance");
        let changed = read_intent(&sufficient);
        assert_eq!(changed.input, "changed");
        assert_eq!(changed.revision, 2);
    }
    let _ = sufficient.close_for_reopen();
    let _ = std::fs::remove_dir_all(&dir);
}
