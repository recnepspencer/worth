//! Closed predecessors retain delivery without retaining their live interests.
use super::*;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
#[test]
fn restored_closed_predecessors_keep_original_successor_refresh_custody() {
    let _guard = checkpoint_recovery_test_guard();
    macro_rules! settle {
        ($owner:ident, $request:ident) => {
            (0..256)
                .find_map(|_| {
                    match $owner.advance(&$request).unwrap_or_else(|stop| {
                        panic!("{} native advance: {stop:?}", stringify!($owner))
                    }) {
                        WorthQueryApplicationOutputDemandProgress::Pending => None,
                        WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
                    }
                })
                .expect("the original native chain owner settles")
        };
    }
    let application = support::install_program::<program::ChainProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut b = request
        .demand(ChainDemand("anchor-b".into()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let mut c = request
        .demand(ChainDemand("anchor-c".into()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    settle!(a, request);
    a.close();
    settle!(b, request);
    b.close();
    settle!(c, request);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows()[0].y, length(1));
    let edit = request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(5),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_7205_u64)
        .execute_performed::<program::ChainProgram, program::ChainRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(edit, worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(_)), "the first source edit performs");
    drop(edit);
    settle!(c, request);
    // seed_cycle authors B.Y=1 and C.Y=10; their handlers preserve Y+1.
    for (key, expected) in [("anchor-a", 6), ("anchor-b", 2), ("anchor-c", 11)] {
        let rows = request
            .query(PlanarOutputRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            rows.rows()[0].value,
            length(expected),
            "authored pre-capture truth {key}"
        );
    }
    let checkpoint = application
        .with_host_advancement(|phase| {
            application
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    c.close();
    drop((source, a, b, c, principal, scope));
    drop(application);

    let restored =
        support::install_program::<program::ChainProgram>(Some(checkpoint), Default::default());
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    // Qualify each restored upstream through a real owner, then close it.
    // Only the original C owner remains live across the dependency edit.
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&restored)
        .unwrap();
    settle!(a, request);
    a.close();
    let mut b = request
        .demand(ChainDemand("anchor-b".into()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&restored)
        .unwrap();
    settle!(b, request);
    b.close();
    let mut c = request
        .demand(ChainDemand("anchor-c".into()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&restored)
        .unwrap();
    let original = settle!(c, request);
    let original_entity = original
        .outputs_of::<PlanarOutputs>()
        .unwrap()
        .entity::<PlanarAnchorOutput<CheckpointSchema>>()
        .unwrap()
        .entity_id();
    for (key, expected) in [("anchor-a", 6), ("anchor-b", 2), ("anchor-c", 11)] {
        let rows = request
            .query(PlanarOutputRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            rows.rows()[0].value,
            length(expected),
            "restored current truth {key}"
        );
    }
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows()[0].y, length(5));
    let edit = request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(6),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_7206_u64)
        .execute_performed::<program::ChainProgram, program::ChainRoot>(
            &restored,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(edit, worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(_)), "the restored source edit performs");
    drop(edit);
    let after_source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        after_source.rows()[0].y,
        length(6),
        "source publication really performed"
    );
    let settled = settle!(c, request);
    assert_eq!(
        settled
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        original_entity
    );
    for (key, expected) in [("anchor-a", 7), ("anchor-b", 2), ("anchor-c", 11)] {
        let rows = request
            .query(PlanarOutputRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            rows.rows()[0].value,
            length(expected),
            "independent post-edit truth {key}"
        );
    }
    c.close();
}

#[test]
fn restored_closed_predecessor_runs_original_dependent_after_source_edit() {
    let _guard = checkpoint_recovery_test_guard();
    macro_rules! settle {
        ($owner:ident, $request:ident) => {
            (0..256)
                .find_map(|_| {
                    match $owner.advance(&$request).unwrap_or_else(|stop| {
                        panic!("{} native advance: {stop:?}", stringify!($owner))
                    }) {
                        WorthQueryApplicationOutputDemandProgress::Pending => None,
                        WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
                    }
                })
                .expect("the same native owner settles")
        };
    }
    macro_rules! edit_source {
        ($app:ident, $request:ident, $before:expr, $after:expr, $key:expr) => {{
            let source = $request.query(PlanarRead { body_key: "anchor-a".into() })
                .execute().unwrap();
            assert_eq!(source.rows()[0].y, length($before));
            let edit = $request.advanced().mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".into(), replacement_y: length($after),
            }).expect_source(source.observed_sources()[0].clone()).idempotency(&$key)
                .execute_performed::<program::ChainProgram, program::ChainRoot>(
                    &$app, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(matches!(edit, worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(_)),
                "the real dependency source edit must perform");
            let after = $request.query(PlanarRead { body_key: "anchor-a".into() })
                .execute().unwrap();
            assert_eq!(after.rows()[0].y, length($after));
        }};
    }
    let application = support::install_program::<program::ChainProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    settle!(a, request);
    a.close();
    let mut b = request
        .demand(ChainDemand("anchor-b".into()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    settle!(b, request);
    edit_source!(application, request, 1, 5, 0x9176_7215_u64);
    settle!(b, request);
    for (key, expected) in [("anchor-a", 6), ("anchor-b", 2)] {
        let output = request
            .query(PlanarOutputRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            output.rows()[0].value,
            length(expected),
            "pre-capture authored truth {key}"
        );
    }
    let checkpoint = application
        .with_host_advancement(|phase| {
            application
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    b.close();
    drop((a, b, principal, scope));
    drop(application);

    let restored =
        support::install_program::<program::ChainProgram>(Some(checkpoint), Default::default());
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&restored)
        .unwrap();
    settle!(a, request);
    a.close();
    let mut b = request
        .demand(ChainDemand("anchor-b".into()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&restored)
        .unwrap();
    let original = settle!(b, request);
    let original_entity = original
        .outputs_of::<PlanarOutputs>()
        .unwrap()
        .entity::<PlanarAnchorOutput<CheckpointSchema>>()
        .unwrap()
        .entity_id();
    for (key, expected) in [("anchor-a", 6), ("anchor-b", 2)] {
        let output = request
            .query(PlanarOutputRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            output.rows()[0].value,
            length(expected),
            "restored authored truth {key}"
        );
    }
    binding::take_decisions("anchor-b");
    edit_source!(restored, request, 5, 6, 0x9176_7216_u64);
    let settled = settle!(b, request);
    let decisions = binding::take_decisions("anchor-b");
    assert!(decisions.iter().any(|values| values.as_slice() == [7]),
        "the original B owner must really decide against current A7, even though B's output stays equal: {decisions:?}");
    assert_eq!(
        settled
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        original_entity
    );
    for (key, expected) in [("anchor-a", 7), ("anchor-b", 2)] {
        let output = request
            .query(PlanarOutputRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            output.rows()[0].value,
            length(expected),
            "post-edit authored truth {key}"
        );
    }
    b.close();
}
