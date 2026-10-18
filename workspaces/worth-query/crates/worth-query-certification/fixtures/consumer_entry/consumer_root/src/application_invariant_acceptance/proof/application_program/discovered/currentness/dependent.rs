//! Empty and deep dependent membership move without changing surviving root sources.
use super::*;

#[test]
fn discovered_currentness_dependent_membership_empty_and_deep() {
    for has_child in [false, true] {
        let world = membership::install();
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        if has_child {
            link!(world, request, "sibling-b", "remote-b", 10_210u64);
        }
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone();
        let outcome = request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length(2),
            })
            .expect_source(source)
            .idempotency(&10_211u64)
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
                WorthQueryOutputDemandControls::new(
                    NonZeroUsize::new(4096).unwrap(),
                    retained_bytes(),
                ),
            )
            .unwrap_or_else(|failure| panic!("native source: {:?}", failure.denial()));
        let aggregate = settled!(world, request, started);
        assert_eq!(aggregate.root_outputs().count(), 3);
        assert_eq!(
            aggregate
                .outputs_for::<MembershipSchema, PlanarOutputToMembershipConnection>()
                .count(),
            usize::from(has_child)
        );
        assert_eq!(
            aggregate
                .outputs_for::<MembershipSchema, PlanarFinalToSummaryConnection>()
                .count(),
            usize::from(has_child)
        );
        if has_child {
            let deep = aggregate
                .outputs_for::<MembershipSchema, PlanarFinalToSummaryConnection>()
                .next()
                .unwrap();
            assert_eq!(deep.0.body_key(), "final:remote-b");
            assert_eq!(
                request
                    .at(aggregate.observation())
                    .query(PlanarOutputRead {
                        body_key: "remote-b".to_owned()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .value,
                length(42)
            );
        }
        request
            .at(aggregate.observation())
            .require_current_discovered_program_outputs(
                &aggregate,
                Some((comparison_work(&world.application)).get()),
            )
            .unwrap();
        let original_members = request
            .query(PlanarDiscoveryRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .source_body_keys
            .clone();
        link!(world, request, "sibling-b", "anchor-b", 10_212u64);
        assert_eq!(
            request
                .query(PlanarDiscoveryRead {
                    body_key: "anchor-a".to_owned()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .source_body_keys,
            original_members
        );
        let edge = request
            .query(PlanarDiscoveryRead {
                body_key: "sibling-b".to_owned(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            edge.rows()[0].source_body_keys.len(),
            usize::from(has_child) + 1
        );
        let current = request.retain_current_read().unwrap();
        for (_, root) in aggregate.root_outputs() {
            request
                .at(&current)
                .require_current_output_demand(
                    root,
                    Some((comparison_work(&world.application)).get()),
                )
                .unwrap();
        }
        stale(
            request
                .at(&current)
                .require_current_discovered_program_outputs(
                    &aggregate,
                    Some((comparison_work(&world.application)).get()),
                ),
        );
        request
            .at(aggregate.observation())
            .require_current_discovered_program_outputs(
                &aggregate,
                Some((comparison_work(&world.application)).get()),
            )
            .unwrap();
        assert_eq!(
            world.application.retained_source_custody_count_for_test(),
            0
        );
    }
}
