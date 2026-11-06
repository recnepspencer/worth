//! Rebuilt candidate content is compared at the real target, with fresh coverage.
use super::super::super::refinement::{self, mutation::RenameExact};
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;

#[test]
fn restored_managed_content_keeps_independent_parent_predicate_revision_requirements() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix_observation::Calls::default());
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
        let populated = request.query(find("A", "sh")).execute().unwrap();
        assert_eq!(
            values(populated.rows()),
            vec![("x".into(), "shared".into())]
        );
        let populated_source = populated.into_result_set_observation().unwrap();
        let disabled = request
            .advanced()
            .mutate(Edit::SetEnabled {
                directory: "A".into(),
                item: "x".into(),
                enabled: false,
            })
            .idempotency(&0x919a_76ffu64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            disabled,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        let original = request.query(find("A", "sh")).execute().unwrap();
        assert!(original.rows().is_empty());
        let source = original.result_set_observation().unwrap().clone();
        assert!(source.has_verified_realized_footprint());
        for (enabled, key) in [(true, 0x919a_7700u64), (false, 0x919a_7702u64)] {
            let changed = request
                .advanced()
                .mutate(Edit::SetEnabled {
                    directory: "A".into(),
                    item: "x".into(),
                    enabled,
                })
                .idempotency(&key)
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(matches!(
                changed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ));
            let head = request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone();
            let outcome = request
                .advanced()
                .mutate(rename("sh", "shared"))
                .expect_result_set(source.clone())
                .idempotency(&(key + 1))
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
            if enabled {
                // The populated observation has independent ordinary revision
                // facts. Restored managed content must not discharge those.
                let populated_outcome = request
                    .advanced()
                    .mutate(rename("sh", "shared"))
                    .expect_result_set(populated_source.clone())
                    .idempotency(&0x919a_7704u64)
                    .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
                let Err(WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Attempt(populated_cause),
                )) = populated_outcome
                else {
                    panic!("ordinary populated field revision must remain required");
                };
                assert_eq!(
                    populated_cause.kind(),
                    WorthQueryApplicationAttemptDenialKind::SourceChanged
                );
            }
            let Err(WorthQueryApplicationRequestMutationDenial::Handler(
                MutationHandlerExecutionDenial::Attempt(cause),
            )) = outcome
            else {
                panic!("actual Parent predicate revisions remain required after restoration");
            };
            assert_eq!(
                cause.kind(),
                WorthQueryApplicationAttemptDenialKind::SourceChanged
            );
            assert_eq!(
                request.retain_current_read().unwrap().selected_commit(),
                &head
            );
        }
        // The handler only reads needle. Both sh observations still carry
        // actual ordinary predicate revisions despite restored managed content.
        assert!(original.rows().is_empty());
        assert_eq!(
            values(request.query(find("A", "sh")).execute().unwrap().rows()),
            values(original.rows())
        );
    });
}

#[test]
fn restored_exact_reject_support_is_equal_only_after_its_hidden_candidate_field_returns() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix_observation::Calls::default());
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
        let exact = || refinement::FindExact {
            directory: "A".into(),
            name: "sha".into(),
        };
        let original = request.query(exact()).execute().unwrap();
        assert!(original.rows().is_empty());
        let source = original.result_set_observation().unwrap().clone();
        assert!(source.has_verified_realized_footprint());
        for (enabled, key) in [(false, 0x919a_7710u64), (true, 0x919a_7712u64)] {
            let changed = request
                .advanced()
                .mutate(Edit::SetEnabled {
                    directory: "A".into(),
                    item: "x".into(),
                    enabled,
                })
                .idempotency(&key)
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(matches!(
                changed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ));
            assert!(request.query(exact()).execute().unwrap().rows().is_empty());
            let head = request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone();
            let outcome = request
                .advanced()
                .mutate(RenameExact(rename("sha", "shared")))
                .expect_result_set(source.clone())
                .idempotency(&(key + 1))
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
            if enabled {
                assert!(matches!(
                    outcome.unwrap(),
                    WorthQueryApplicationMutationOutcome::Committed { .. }
                ));
            } else {
                let Err(WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Attempt(cause),
                )) = outcome
                else {
                    panic!("identical exact DTO cannot hide changed rejected support");
                };
                assert_eq!(
                    cause.kind(),
                    WorthQueryApplicationAttemptDenialKind::SourceChanged
                );
                assert_eq!(
                    request.retain_current_read().unwrap().selected_commit(),
                    &head
                );
            }
        }
        assert!(original.rows().is_empty());
    });
}
