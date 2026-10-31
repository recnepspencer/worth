//! Genuine old expectations distinguish candidate influence from the whole Source.
use super::super::{authoring::Edit, handler::DecisionRead, schema::Catalog};
use super::*;

#[test]
fn realized_footprint_preserves_irrelevant_source_edits_and_refuses_changed_projected_inputs() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let original = request.query(find("A", "sh")).execute().unwrap();
        assert_eq!(values(original.rows()), vec![("x".into(), "shared".into())]);
        let source = original.result_set_observation().unwrap().clone();
        assert!(source.has_verified_realized_footprint());
        insert(
            &world,
            &principal,
            &scope,
            "irrelevant",
            "zz-other",
            0x919a_7000,
        );
        let committed = request
            .advanced()
            .mutate(rename("sh", "free"))
            .expect_result_set(source)
            .idempotency(&0x919a_7001u64)
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
        assert_eq!(values(original.rows()), vec![("x".into(), "shared".into())]);
        let fresh = request.query(find("A", "fr")).execute().unwrap();
        assert_eq!(values(fresh.rows()), vec![("x".into(), "free".into())]);
        let source = fresh.into_result_set_observation().unwrap();
        let changed = request
            .advanced()
            .mutate(Edit::SetEnabled {
                directory: "A".into(),
                item: "x".into(),
                enabled: false,
            })
            .idempotency(&0x919a_7002u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            changed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        // The provider candidate key/name remains the same. The real Parent's
        // tracked enabled predicate changes its membership and must be retained.
        assert!(request
            .query(find("A", "fr"))
            .execute()
            .unwrap()
            .rows()
            .is_empty());
        let head = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let refused = request
            .advanced()
            .mutate(rename("fr", "wrong"))
            .expect_result_set(source)
            .idempotency(&0x919a_7003u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
        let Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Attempt(cause),
        )) = refused
        else {
            panic!("changed projected influence must refuse at source admission");
        };
        assert_eq!(
            cause.kind(),
            WorthQueryApplicationAttemptDenialKind::SourceChanged
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &head
        );
    });
}

#[test]
fn realized_footprint_empty_expectation_survives_nonmatches_but_keeps_future_entrants() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix::Calls::default());
        let world = host::install_decision_read(
            calls.clone(),
            Arc::new(rule::Calls::new(calls)),
            DecisionRead::Empty,
        );
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let empty = request.query(find("A", "needle")).execute().unwrap();
        assert!(empty.rows().is_empty());
        let original = empty.result_set_observation().unwrap().clone();
        assert!(original.has_verified_realized_footprint());
        insert(
            &world,
            &principal,
            &scope,
            "irrelevant",
            "other",
            0x919a_7100,
        );
        let committed = request
            .advanced()
            .mutate(rename("needle", "renamed"))
            .expect_result_set(original)
            .idempotency(&0x919a_7101u64)
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
        assert!(empty.rows().is_empty());
        let fresh = request.query(find("A", "needle")).execute().unwrap();
        assert!(fresh.rows().is_empty());
        let source = fresh.into_result_set_observation().unwrap();
        insert(&world, &principal, &scope, "entrant", "needle", 0x919a_7102);
        assert_eq!(
            values(request.query(find("A", "needle")).execute().unwrap().rows()),
            vec![("entrant".into(), "needle".into())]
        );
        let head = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let refused = request
            .advanced()
            .mutate(rename("needle", "wrong"))
            .expect_result_set(source)
            .idempotency(&0x919a_7103u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
        let Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Attempt(cause),
        )) = refused
        else {
            panic!("the old empty-range obligation must refuse the actual entrant");
        };
        assert_eq!(
            cause.kind(),
            WorthQueryApplicationAttemptDenialKind::SourceChanged
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &head
        );
    });
}

fn insert(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
    identity: &str,
    name: &str,
    key: u64,
) {
    let committed = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world())
        .advanced()
        .mutate(Edit::Item {
            directory: "A".into(),
            identity: identity.into(),
            name: name.into(),
        })
        .idempotency(&key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            committed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{:?}",
        committed.commit_outcome()
    );
}

mod atomic;

#[test]
fn realized_footprint_keeps_two_distinct_probe_obligations_at_the_same_source_revision() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix::Calls::default());
        let world = host::install_decision_read(
            calls.clone(),
            Arc::new(rule::Calls::new(calls)),
            DecisionRead::TwoProbes,
        );
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let source = request
            .query(find("A", "sh"))
            .execute()
            .unwrap()
            .into_result_set_observation()
            .unwrap();
        assert!(source.has_verified_realized_footprint());
        // The old expectation is sh; the actual committed handler reads sha at
        // this same Source revision. A Source-only key would collide the two.
        let result = request
            .advanced()
            .mutate(rename("sh", "shared"))
            .expect_result_set(source)
            .idempotency(&0x919a_7400u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                result,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{:?}",
            result.commit_outcome()
        );
    });
}

mod exact;
