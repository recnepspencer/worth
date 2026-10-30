//! Independently admitted public queries cannot widen the owning operation's Parent read.
use super::super::handler::DecisionRead;
use super::*;

#[test]
fn decision_index_requires_the_declared_parent_and_the_original_operation_scope() {
    lifecycle::run(|| {
        for mode in [DecisionRead::OtherScope, DecisionRead::Undeclared] {
            let provider = Arc::new(prefix::Calls::default());
            let rules = Arc::new(rule::Calls::new(Arc::clone(&provider)));
            let world =
                host::install_decision_read(Arc::clone(&provider), Arc::clone(&rules), mode);
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
            let a = request
                .query(find("A", "sh"))
                .open_index::<prefix::PrefixIndex>()
                .unwrap();
            let b = request
                .query(find("B", "sh"))
                .open_index::<prefix::PrefixIndex>()
                .unwrap();
            assert_eq!(
                values(request.query(find("B", "sh")).execute().unwrap().rows()),
                expected("B", "shared", "sh")
            );
            assert_eq!(
                values(
                    request
                        .query(query::ReadSource {
                            directory: "A".into()
                        })
                        .execute()
                        .unwrap()
                        .rows()
                ),
                expected("A", "shared", "")
            );
            let head = request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone();
            let source = request.query(find("A", "sh")).execute().unwrap();
            let calls = provider.probe_validations.load(Ordering::SeqCst);
            rules.records.lock().unwrap().clear();
            let result = request
                .advanced()
                .mutate(rename("sh", "wrong"))
                .expect_result_set(source.result_set_observation().unwrap().clone())
                .idempotency(&0x919320u64)
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
            let Err(WorthQueryApplicationRequestMutationDenial::Handler(
                MutationHandlerExecutionDenial::Handler(cause),
            )) = result
            else {
                panic!("the declared handler read must reach its own typed refusal: {result:?}")
            };
            match mode {
                DecisionRead::OtherScope => {
                    let cause = cause
                        .downcast::<WorthQueryManagedIndexDenial>()
                        .expect("actual managed scope refusal");
                    assert!(
                        matches!(
                            cause.cause(),
                            WorthQueryManagedIndexDenialCause::Refused(
                                WorthQueryManagedIndexRefusal::ForeignAssociation
                            )
                        ),
                        "{cause:?}"
                    );
                }
                DecisionRead::Undeclared => {
                    let cause = cause
                        .downcast::<WorthQueryInvariantDecisionPlanDenial>()
                        .expect("actual undeclared Parent refusal");
                    assert_eq!(
                        cause.kind(),
                        WorthQueryInvariantDecisionPlanDenialKind::UndeclaredDecisionTarget
                    );
                }
                DecisionRead::Declared => unreachable!(),
            }
            assert_eq!(
                provider.probe_validations.load(Ordering::SeqCst),
                calls,
                "binding/scope refusal precedes every provider query"
            );
            assert!(rules.records.lock().unwrap().is_empty());
            assert_eq!(
                request.retain_current_read().unwrap().selected_commit(),
                &head
            );
            assert_eq!(
                values(request.query(find("A", "")).execute().unwrap().rows()),
                expected("A", "shared", "")
            );
            a.close();
            b.close();
        }
    });
}
