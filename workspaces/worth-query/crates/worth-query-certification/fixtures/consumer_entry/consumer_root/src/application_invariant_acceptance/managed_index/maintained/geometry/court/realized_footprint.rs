//! Indirect Source inputs participate in real retained expectation comparison.
use super::super::refresh;
use super::*;

#[test]
fn realized_footprint_geometry_preserves_nonmatching_motion_and_refuses_candidate_endpoint_change()
{
    lifecycle::run(|| {
        let boxes = Arc::new(provider::Calls::default());
        let prefixes = Arc::new(prefix::Calls::default());
        let world = host::install_geometry(
            prefixes.clone(),
            Arc::new(prefix_rule::Calls::new(prefixes)),
            Configuration::new(boxes),
        );
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        setup(&world, &principal, &scope);
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let original = request.query(find("B", 5)).execute().unwrap();
        assert_eq!(
            values(original.rows()),
            expected("B", 10, find("B", 5).probe)
        );
        let source = original.result_set_observation().unwrap().clone();
        assert!(source.has_verified_realized_footprint());
        commit(
            &world,
            &principal,
            &scope,
            Edit::Move {
                directory: "catalog".into(),
                vertex: "v3".into(),
                x: 61,
                y: 0,
            },
            0x919a_7300,
        );
        let committed = request
            .advanced()
            .mutate(refresh::Refresh {
                probe: find("B", 5).probe,
                directory: "B".into(),
                note: "noncandidate changed".into(),
            })
            .expect_result_set(source)
            .idempotency(&0x919a_7301u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                committed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{:?}",
            committed.commit_outcome()
        );
        assert_eq!(
            values(original.rows()),
            expected("B", 10, find("B", 5).probe)
        );
        let current = request.query(find("B", 5)).execute().unwrap();
        let source = current.into_result_set_observation().unwrap();
        commit(
            &world,
            &principal,
            &scope,
            Edit::Move {
                directory: "catalog".into(),
                vertex: "v1".into(),
                x: 9,
                y: 0,
            },
            0x919a_7302,
        );
        let head = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let refused = request
            .advanced()
            .mutate(refresh::Refresh {
                probe: find("B", 5).probe,
                directory: "B".into(),
                note: "wrong".into(),
            })
            .expect_result_set(source)
            .idempotency(&0x919a_7303u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
        let Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Attempt(cause),
        )) = refused
        else {
            panic!("indirect projected endpoint changes must refuse the old expectation");
        };
        assert_eq!(
            cause.kind(),
            WorthQueryApplicationAttemptDenialKind::SourceChanged
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &head
        );
        assert_eq!(
            values(request.query(find("B", 5)).execute().unwrap().rows()),
            expected("B", 9, find("B", 5).probe)
        );
    });
}
