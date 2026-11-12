pub(crate) use worth_query_testkit_certification_canonical::facade::certification::canonical_transition::related::companion_transition::capture::*;

#[test]
fn companion_post_ack_capture_refusal_repairs_same_capsule_with_historical_roster() {
    let (original, settlement) = original(model::authored_value());
    let ids = companion_ids(&settlement);
    let source_id = old::reads::source_id(&original);
    let checkpoint = old::host::capture(&original);
    let held = checkpoint.clone();
    let bytes = checkpoint.bytes().to_vec();
    successor::output::reset_decodes();
    let mut callbacks = 0;
    let pending = worth_query_testkit_host::facade::host::checkpoint_capture::stopped(|phase| {
        callbacks = 0;
        installation::WorthQueryInterimCheckpointRestore::rostered_program_with_transition_in_advancement(
            phase,
            new::host::program::<new::program::TargetProgram>(),
            installation::WorthQueryApplicationProgramRoster::new(),
            successor::schema::CanonicalSchema::declaration().unwrap(),
            ((),),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            successor::host::limits(),
            checkpoint.clone(),
            installation::WorthQueryCheckpointProgramPredecessor::new(
                &old::host::program::<old::program::SourceProgram>()
                    .revision()
                    .to_string(),
            )
            .unwrap(),
            installation::WorthQueryCheckpointTransitionResources::bounded(65_536, 32, 65_536)
                .unwrap()
                .with_capture_memory_limit(0),
            phase.execution_request().allocation_policy(),
            |_, _| {
                callbacks += 1;
                Ok(())
            },
        )
    });
    let Some(installation::WorthQueryCheckpointCaptureDenial::Execution(cause)) =
        pending.capture_denial()
    else {
        panic!(
            "complete capture keeps the original typed stop: {:?}",
            pending.capture_denial()
        );
    };
    let worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
        kind: worth_query_execution::facade::primary_graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::MemoryLimit {
                requested, admitted: 0,
                level: worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy,
            }, partition_identity: None, policy_ancestor: Some(0),
        }
    } = cause.kind() else { panic!("exact zero-memory capture cause: {cause:?}"); };
    assert!(requested > 0);
    assert_eq!(callbacks, 1);
    assert_eq!(successor::output::decodes(), 0);
    // An unfunded original request returns the same acknowledged capsule again.
    let zero = worth_query_testkit_host::facade::host::checkpoint_capture::lease(0);
    let pending = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&zero))
        .unwrap_err();
    assert!(pending.capture_denial().is_some());
    let funded =
        worth_query_testkit_host::facade::host::checkpoint_capture::lease(64 * 1024 * 1024);
    let repaired = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&funded))
        .unwrap();
    assert_eq!(callbacks, 1, "same-capsule repair never reauthors");
    assert_eq!(held.bytes(), bytes);
    let target = new::host::restore::<new::program::TargetProgram>(
        repaired,
        installation::WorthQueryApplicationProgramRoster::new(),
    );
    assert_eq!(new::reads::source_id(&target), source_id);
    assert_eq!(successor::output::decodes(), 0);
    assert_target(&target, ids);
}
