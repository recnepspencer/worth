//! Retired original roots remain distinct from an originally empty demand set.
use super::*;
#[test]
fn discovered_currentness_all_native_retired_keeps_original_discovery() {
    world!(world, request);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_213u64)
        .execute_performed_discovered::<MembershipProgram, FanoutRoot>(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
        panic!("source publishes once")
    };
    let receipt = performed.receipt().clone();
    drop(performed);
    for (body, key) in [("sibling-b", 10_214u64), ("remote-b", 10_215u64)] {
        let source = request
            .query(PlanarRead {
                body_key: body.to_owned(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone();
        assert!(matches!(
            request
                .advanced()
                .mutate(PlanarEdit(PlanarMutation {
                    scope_key: body.to_owned(),
                    operation: PlanarOperation::Adjust(vec![PlanarAdjustment {
                        body_key: body.to_owned(),
                        replacement_y: length(5)
                    }])
                }))
                .expect_source(source)
                .idempotency(&key)
                .execute_in_program(
                    &world.application,
                    ExecutionAllocationPolicy::SystemAllocation
                )
                .unwrap(),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    }
    let mut recovered = request
        .recover_discovered_required_outputs::<MembershipProgram, FanoutRoot>(
            &world.application,
            &receipt,
            WorthQueryOutputDemandControls::new(NonZeroUsize::new(4096).unwrap(), retained_bytes()),
        )
        .unwrap();
    let aggregate = settle(
        || match recovered.advance(&world.application, &request).unwrap() {
            WorthQueryDiscoveredProgramOutputProgress::Pending => None,
            WorthQueryDiscoveredProgramOutputProgress::Settled(aggregate) => Some(aggregate),
        },
    );
    assert_eq!(aggregate.root_outputs().count(), 0);
    assert_eq!(aggregate.output_count(), 0);
    assert_eq!(aggregate.superseded_roots().count(), 3);
    let current = request.retain_current_read().unwrap();
    request
        .at(&current)
        .require_current_discovered_program_outputs(&aggregate, comparison_work(&world.application))
        .unwrap();
    link!(world, request, "anchor-a", "anchor-b", 10_216u64);
    let current = request.retain_current_read().unwrap();
    stale(
        request
            .at(&current)
            .require_current_discovered_program_outputs(
                &aggregate,
                comparison_work(&world.application),
            ),
    );
    assert_eq!(aggregate.superseded_roots().count(), 3);
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}
