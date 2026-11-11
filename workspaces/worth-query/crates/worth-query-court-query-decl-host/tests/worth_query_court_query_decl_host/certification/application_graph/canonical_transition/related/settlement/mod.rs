pub(crate) use worth_query_testkit_certification_canonical::facade::certification::canonical_transition::related::settlement::*;

#[test]
fn related_acknowledged_successor_repairs_capture_without_reauthoring_or_old_decode() {
    let original = host::install(model::authored_value()).unwrap();
    let _old = host::produce(&original);
    let source_id = host::source_id(&original);
    let checkpoint = host::capture(&original);
    let retained = checkpoint.clone();
    let bytes = checkpoint.bytes().to_vec();
    let mut callbacks = 0;
    successor::output::reset_decodes();
    let pending = worth_query_testkit_host::facade::host::checkpoint_capture::stopped(|phase| {
        callbacks = 0;
        installation::WorthQueryInterimCheckpointRestore::rostered_program_with_transition_in_advancement(
            phase,
            successor::host::program(),
            installation::WorthQueryApplicationProgramRoster::new()
                .support(successor::companion::host::program::<
                    successor::companion::program::TargetProgram,
                >()),
            successor::schema::CanonicalSchema::declaration().unwrap(),
            ((),),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            successor::host::limits(),
            checkpoint.clone(),
            installation::WorthQueryCheckpointProgramPredecessor::new(
                &host::program().revision().to_string(),
            )
            .unwrap(),
            installation::WorthQueryCheckpointTransitionResources::bounded(8192, 32, 65_536)
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
    let repaired = worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(64 * 1024 * 1024), Some(8_000_000)),
        ),
        |phase| {
            pending
                .repair_to_checkpoint(
                    &phase,
                    installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        },
    )
    .unwrap();
    assert_eq!(callbacks, 1);
    assert_eq!(retained.bytes(), bytes);
    let target = successor::host::restore(repaired);
    assert_eq!(successor::host::source_id(&target), source_id);
    assert_eq!(successor::output::decodes(), 0);
    let _new = successor::host::produce(&target);
    journey::assert_target_pages(&target, false);
}
