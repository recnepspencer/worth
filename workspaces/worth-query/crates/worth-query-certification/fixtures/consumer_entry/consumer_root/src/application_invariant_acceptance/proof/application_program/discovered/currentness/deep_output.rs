//! A moved dependent source is invisible to both surviving roots and discovery membership.
use super::*;

#[test]
fn discovered_currentness_all_retained_dependent_settlements() {
    let world = membership::install();
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    link!(world, request, "sibling-a", "sibling-b", 10_217u64);
    link!(world, request, "sibling-b", "remote-b", 10_218u64);
    let source = request
        .query(PlanarRead {
            body_key: "sibling-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let outcome = request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "sibling-a".to_owned(),
            replacement_y: length(22),
        })
        .expect_source(source)
        .idempotency(&10_219u64)
        .execute_performed_discovered::<MembershipProgram, MembershipRoot>(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
        panic!("native source publishes once")
    };
    let mut started = performed
        .start_required_outputs(
            &world.application,
            &request,
            WorthQueryOutputDemandControls::default(),
        )
        .unwrap_or_else(|failure| panic!("native source: {:?}", failure.denial()));
    let aggregate = settled!(world, request, started);
    assert_eq!(aggregate.root_outputs().count(), 1);
    assert_eq!(
        aggregate
            .outputs_for::<MembershipSchema, PlanarOutputToMembershipConnection>()
            .count(),
        1
    );
    assert_eq!(
        aggregate
            .outputs_for::<MembershipSchema, PlanarFinalToSummaryConnection>()
            .count(),
        1
    );
    let historical = aggregate.observation();
    request
        .at(historical)
        .require_current_discovered_program_outputs(&aggregate)
        .unwrap();
    let final_value = request
        .query(PlanarOutputRead {
            body_key: "final:remote-b".to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .value;
    assert_eq!(final_value, length(43));
    assert_eq!(
        aggregate.output_count(),
        3,
        "one root, one middle, one final leaf"
    );
    let leaf_key = "final:final:remote-b:c";
    let leaf_source = request
        .query(PlanarRead {
            body_key: leaf_key.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(leaf_source.rows()[0].y, length(46));
    let leaf_expectation = leaf_source.observed_sources()[0].clone();
    drop(leaf_source);
    assert!(matches!(
        request
            .advanced()
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: leaf_key.to_owned(),
                operation: PlanarOperation::Adjust(vec![PlanarAdjustment {
                    body_key: leaf_key.to_owned(),
                    replacement_y: length(47),
                }]),
            }))
            .expect_source(leaf_expectation)
            .idempotency(&10_221u64)
            .execute_in_program(
                &world.application,
                ExecutionAllocationPolicy::SystemAllocation
            )
            .unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let leaf_current = request.retain_current_read().unwrap();
    for (_, root) in aggregate.root_outputs() {
        request
            .at(&leaf_current)
            .require_current_output_demand(root)
            .unwrap();
    }
    let (_, middle) = aggregate
        .outputs_for::<MembershipSchema, PlanarOutputToMembershipConnection>()
        .next()
        .unwrap();
    request
        .at(&leaf_current)
        .require_current_output_demand(middle)
        .unwrap();
    let (_, leaf) = aggregate
        .outputs_for::<MembershipSchema, PlanarFinalToSummaryConnection>()
        .next()
        .unwrap();
    stale(
        request
            .at(&leaf_current)
            .require_current_output_demand(leaf),
    );
    stale(
        request
            .at(&leaf_current)
            .require_current_discovered_program_outputs(&aggregate),
    );
    request
        .at(historical)
        .require_current_discovered_program_outputs(&aggregate)
        .unwrap();
    let source = request
        .query(PlanarRead {
            body_key: "remote-b".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    assert!(matches!(
        request
            .advanced()
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: "remote-b".to_owned(),
                operation: PlanarOperation::Adjust(vec![PlanarAdjustment {
                    body_key: "remote-b".to_owned(),
                    replacement_y: length(5)
                }])
            }))
            .expect_source(source)
            .idempotency(&10_220u64)
            .execute_in_program(
                &world.application,
                ExecutionAllocationPolicy::SystemAllocation
            )
            .unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "remote-b".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(5)
    );
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "final:remote-b".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        final_value,
        "currentness checks do not refresh the original handler or output"
    );
    for (key, expected) in [("sibling-a", "sibling-b"), ("sibling-b", "remote-b")] {
        assert_eq!(
            request
                .query(PlanarDiscoveryRead {
                    body_key: key.to_owned()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .source_body_keys,
            [expected.to_owned()]
        );
    }
    let current = request.retain_current_read().unwrap();
    for (_, root) in aggregate.root_outputs() {
        request
            .at(&current)
            .require_current_output_demand(root)
            .unwrap();
    }
    let (_, dependent) = aggregate
        .outputs_for::<MembershipSchema, PlanarOutputToMembershipConnection>()
        .next()
        .unwrap();
    stale(
        request
            .at(&current)
            .require_current_output_demand(dependent),
    );
    stale(
        request
            .at(&current)
            .require_current_discovered_program_outputs(&aggregate),
    );
    request
        .at(historical)
        .require_current_discovered_program_outputs(&aggregate)
        .unwrap();
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}
