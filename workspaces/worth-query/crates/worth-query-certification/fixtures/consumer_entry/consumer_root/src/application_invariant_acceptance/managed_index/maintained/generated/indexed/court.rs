//! Real managed Source epochs cross ordinary producer reuse and fresh delivery.
use super::super::super::{authoring::Edit, host, query, rule, schema::Catalog};
use super::{producer, program};
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use crate::application_invariant_acceptance::{authentication, managed_index::lifecycle};
use std::sync::Arc;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_host::facade::{application_entry::*, primary_graph::*};
type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, Catalog>;
type Settled = WorthQuerySettledProgramOutputOperation<Catalog, host::Program, program::Root>;
fn find(directory: &str, prefix: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: directory.into(),
        prefix: prefix.into(),
    }
}
fn settle(world: &host::Host, request: &Request<'_, '_, '_>, directory: &str) -> Settled {
    let mut operation = request
        .demand(producer::Demand {
            directory: directory.into(),
        })
        .retain_program_outputs::<host::Program, program::Root>(world)
        .unwrap_or_else(|stop| panic!("managed tree construction: {:?}", stop.denial()));
    for _ in 0..64 {
        operation = match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => return settled,
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("managed tree refused: {denial:?}")
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("managed tree terminal: {:?}", terminal.denial())
            }
        };
    }
    panic!("one genuine producer settles within the finite fixture bound")
}
fn edit(world: &host::Host, request: &Request<'_, '_, '_>, input: Edit, key: u64) {
    let outcome = request
        .advanced()
        .mutate(input)
        .idempotency(&key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{:?}",
        outcome.commit_outcome()
    );
}
#[test]
fn managed_source_reuses_real_output_after_self_publication_and_irrelevant_edit() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix_observation::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
        let original_receipt = {
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
            let source = request.query(find("A", "sh")).execute().unwrap();
            assert_eq!(
                source
                    .rows()
                    .iter()
                    .map(|row| row.identity.as_str())
                    .collect::<Vec<_>>(),
                ["x"]
            );
            assert!(source
                .result_set_observation()
                .unwrap()
                .has_verified_realized_footprint());
            let exact_current = request.retain_current_read().unwrap();
            let selected = request
                .at(&exact_current)
                .query(find("A", "sh"))
                .execute()
                .unwrap();
            assert_eq!(selected.rows(), source.rows());
            assert!(selected
                .result_set_observation()
                .unwrap()
                .has_verified_realized_footprint());
            let initial = settle(&world, &request, "A");
            assert_eq!(
                initial.settlement().root_producer_contacts_in_this_demand(),
                1
            );
            assert!(initial.settlement().root_receipt().is_some());
            let original_receipt = initial.settlement().root_receipt().unwrap().clone();
            let output = initial
                .read_current(&request, find("A", "produced:"))
                .unwrap();
            assert_eq!(output.rows()[0].name, "produced:shared");
            // The generated row is an actual maintained Source insertion in both
            // directories. Its name is outside sh, so it cannot invalidate x.
            assert_eq!(
                request.query(find("A", "sh")).execute().unwrap().rows(),
                source.rows()
            );
            original_receipt
        };
        // Original authority is gone. A fresh actual request admits both source
        // discovery and final output disclosure; no old receipt supplies READ.
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
        for stage in 0..2 {
            if stage == 1 {
                edit(
                    &world,
                    &request,
                    Edit::Item {
                        directory: "A".into(),
                        identity: "unrelated".into(),
                        name: "zz-other".into(),
                    },
                    0x919a_8000,
                );
            }
            let head = request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone();
            let reused = settle(&world, &request, "A");
            assert_eq!(
                reused.settlement().root_settlement().posture(),
                WorthQueryOutputSettlementPosture::Performed
            );
            assert!(
                reused
                    .settlement()
                    .root_receipt()
                    .unwrap()
                    .is_same_authoritative_commit(&original_receipt),
                "reuse retains the initial committed receipt; no new producer receipt is minted"
            );
            assert_eq!(
                reused.settlement().root_producer_contacts_in_this_demand(),
                0
            );
            let output = reused
                .read_current(&request, find("A", "produced:"))
                .unwrap();
            assert_eq!(output.rows().len(), 1);
            assert_eq!(
                (&*output.rows()[0].identity, &*output.rows()[0].name),
                ("managed-x-shared", "produced:shared")
            );
            assert_eq!(
                request.retain_current_read().unwrap().selected_commit(),
                &head
            );
        }
    });
}
#[test]
fn managed_reused_output_refuses_changed_candidate_and_future_entrant() {
    lifecycle::run(|| {
        for changed in 0..3 {
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
            let initial = settle(&world, &request, "A");
            assert!(initial.settlement().root_receipt().is_some());
            drop(initial);
            let reused = settle(&world, &request, "A");
            assert_eq!(
                reused.settlement().root_settlement().posture(),
                WorthQueryOutputSettlementPosture::Performed
            );
            if changed == 2 {
                let source = request
                    .query(find("A", "sh"))
                    .execute()
                    .unwrap()
                    .into_result_set_observation()
                    .unwrap();
                let outcome = request
                    .advanced()
                    .mutate(super::super::super::mutation::Rename {
                        directory: "A".into(),
                        prefix: "sh".into(),
                        item: "x".into(),
                        replacement: "sham".into(),
                    })
                    .expect_result_set(source)
                    .idempotency(&0x919a_8101u64)
                    .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
                    .unwrap();
                assert!(matches!(
                    outcome,
                    WorthQueryApplicationMutationOutcome::Committed { .. }
                ));
                assert_eq!(
                    request
                        .query(find("A", "sh"))
                        .execute()
                        .unwrap()
                        .rows()
                        .iter()
                        .map(|row| row.identity.as_str())
                        .collect::<Vec<_>>(),
                    ["x"],
                    "candidate membership is unchanged while full projected name changes"
                );
            } else {
                let input = if changed == 1 {
                    Edit::Item {
                        directory: "A".into(),
                        identity: "entrant".into(),
                        name: "sh-entrant".into(),
                    }
                } else {
                    Edit::SetEnabled {
                        directory: "A".into(),
                        item: "x".into(),
                        enabled: false,
                    }
                };
                edit(&world, &request, input, 0x919a_8100);
            }
            let head = request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone();
            let delivery = reused.deliver_current(&request);
            assert!(
                matches!(&delivery, Err(WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause))) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded),
                "old complete membership must be superseded: {:?}",
                delivery.as_ref().err()
            );
            assert!(
                matches!(reused.read_current(&request, find("A", "produced:")), Err(WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause))) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded)
            );
            assert_eq!(request.query(find("A", "produced:")).execute().unwrap().rows()[0].name, "produced:shared", "the produced row remains normally readable; Source currentness alone refuses delivery");
            assert_eq!(
                request.retain_current_read().unwrap().selected_commit(),
                &head
            );
            assert_eq!(
                reused.settlement().root_producer_contacts_in_this_demand(),
                0
            );
            if changed == 2 {
                initial_successor::refuse(&world, &request);
            }
        }
    });
}

#[test]
fn managed_source_producer_refuses_foreign_operation_scope_even_for_shared_row() {
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
        let a = request.query(find("A", "sh")).execute().unwrap();
        let b = request.query(find("B", "sh")).execute().unwrap();
        assert_eq!(a.rows()[0].coordinate, b.rows()[0].coordinate);
        assert!(b
            .result_set_observation()
            .unwrap()
            .has_verified_realized_footprint());
        let head = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let mut operation = request
            .demand(producer::Demand {
                directory: "B".into(),
            })
            .retain_program_outputs::<host::Program, program::Root>(&world)
            .unwrap_or_else(|stop| panic!("scope tree construction: {:?}", stop.denial()));
        let mut refused = false;
        for _ in 0..64 {
            operation = match operation.advance(&request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                    assert_scope_refusal(&denial);
                    refused = true;
                    break;
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                    assert_scope_refusal(terminal.denial());
                    refused = true;
                    break;
                }
                WorthQueryProgramOutputOperationAdvance::Settled(_) => {
                    panic!("B Source cannot authorize an A operation even with the same actual row")
                }
            };
        }
        assert!(
            refused,
            "scope association reaches its actual finite refusal"
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &head
        );
        assert!(request
            .query(find("A", "produced:"))
            .execute()
            .unwrap()
            .rows()
            .is_empty());
    });
}
fn assert_scope_refusal(denial: &WorthQueryProgramOutputAdvanceDenial) {
    let cause = match denial {
        WorthQueryProgramOutputAdvanceDenial::Outputs(
            WorthQueryRequiredOutputPreparationDenial::DemandExecution(cause),
        ) => cause,
        WorthQueryProgramOutputAdvanceDenial::Outputs(
            WorthQueryRequiredOutputPreparationDenial::Demand(
                WorthQueryApplicationOutputDemandDenial::Demand(cause),
            ),
        ) => cause,
        other => panic!("wrong scope admission refusal: {other:?}"),
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOutputDemandDenialKind::ProducerUnavailable
    );
}

mod initial_successor;

mod membership;
