//! Genuine governed observations retain hidden influence through atomic expectations.
use super::super::{super::mutation, expectation};
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
#[test]
fn governed_realized_expectation_preserves_irrelevant_edits_and_rejects_changed_influence() {
    lifecycle::run(|| {
        for change in 0..4 {
            let calls = Arc::new(prefix_observation::Calls::default());
            let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls.clone())));
            let scope = authentication::request_scope();
            let principal = authentication::block_on(
                authentication::admit(world.installed_schema()).authenticate(
                    authentication::LocalCredential::issued_for_model_owner(),
                    &scope,
                ),
            )
            .unwrap();
            create(
                &world,
                &principal,
                &scope,
                "governed-original",
                "shared",
                0x919b_1000,
            );
            let request = world
                .runtime()
                .request(&principal, &scope)
                .on_branch(world.current_world());
            if change == 3 {
                create(
                    &world,
                    &principal,
                    &scope,
                    "governed-rejected",
                    "shared-left",
                    0x919b_1003,
                );
            }
            let original = request
                .query(query::exact::Find {
                    directory: "catalog".into(),
                    prefix: "shared".into(),
                })
                .execute_governed(
                    access::ReadBoxes::reference(),
                    access::InspectBoxes::reference(),
                    purpose(),
                )
                .unwrap();
            assert_eq!(original.rows().len(), 1);
            assert!(matches!(
                original.rows()[0].name,
                WorthQueryApplicationDisclosed::Omitted(_)
            ));
            let source = original.result_set_observation().unwrap().clone();
            assert!(source.has_verified_realized_footprint());
            match change {
                0 => create(
                    &world,
                    &principal,
                    &scope,
                    "unrelated",
                    "zz-other",
                    0x919b_1001,
                ),
                1 => {
                    let outcome = request
                        .advanced()
                        .mutate(Edit::SetEnabled {
                            directory: "catalog".into(),
                            item: "governed-original".into(),
                            enabled: false,
                        })
                        .idempotency(&0x919b_1001u64)
                        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
                        .unwrap();
                    assert!(matches!(
                        outcome,
                        WorthQueryApplicationMutationOutcome::Committed { .. }
                    ));
                }
                3 => {
                    let observed = request
                        .query(super::super::super::query::FindPrefix {
                            directory: "catalog".into(),
                            prefix: "shared".into(),
                        })
                        .execute()
                        .unwrap()
                        .into_result_set_observation()
                        .unwrap();
                    let outcome = request
                        .advanced()
                        .mutate(mutation::Rename {
                            directory: "catalog".into(),
                            prefix: "shared".into(),
                            item: "governed-rejected".into(),
                            replacement: "shared-right".into(),
                        })
                        .expect_result_set(observed)
                        .idempotency(&0x919b_1001u64)
                        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
                        .unwrap();
                    assert!(matches!(
                        outcome,
                        WorthQueryApplicationMutationOutcome::Committed { .. }
                    ));
                    let same = request
                        .query(query::exact::Find {
                            directory: "catalog".into(),
                            prefix: "shared".into(),
                        })
                        .execute_governed(
                            access::ReadBoxes::reference(),
                            access::InspectBoxes::reference(),
                            purpose(),
                        )
                        .unwrap();
                    assert_eq!(
                        same.rows(),
                        original.rows(),
                        "a rejected body changes while the exact disclosed DTO is identical"
                    );
                }
                _ => create(&world, &principal, &scope, "entrant", "shared", 0x919b_1001),
            }
            let head = request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone();
            let result = request
                .advanced()
                .mutate(expectation::RenameExact(mutation::Rename {
                    directory: "catalog".into(),
                    prefix: "shared".into(),
                    item: "governed-original".into(),
                    replacement: "free".into(),
                }))
                .expect_result_set(source)
                .idempotency(&0x919b_1002u64)
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
            if change == 0 {
                let outcome = result.unwrap();
                assert!(
                    matches!(
                        outcome,
                        WorthQueryApplicationMutationOutcome::Committed { .. }
                    ),
                    "{:?}",
                    outcome.commit_outcome()
                );
                let current = request
                    .query(query::exact::Find {
                        directory: "catalog".into(),
                        prefix: "free".into(),
                    })
                    .execute_governed(
                        access::ReadBoxes::reference(),
                        access::InspectBoxes::reference(),
                        purpose(),
                    )
                    .unwrap();
                assert_eq!(current.rows()[0].identity, "governed-original");
                assert!(matches!(
                    current.rows()[0].name,
                    WorthQueryApplicationDisclosed::Omitted(_)
                ));
            } else {
                let Err(WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Attempt(cause),
                )) = result
                else {
                    panic!("changed actual governed influence must refuse Source expectation");
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
            assert!(matches!(
                original.rows()[0].name,
                WorthQueryApplicationDisclosed::Omitted(_)
            ));
            assert_eq!(original.rows()[0].identity, "governed-original");
        }
    });
}
