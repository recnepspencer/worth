//! Actual required-set checks include complete row and set walk facts.
use super::super::{commit, program};
use super::*;
use crate::publication_boundary::{
    discovery_currentness::{
        WorthQueryRequiredSetCurrentnessDenial, WorthQueryRetainedQueryDependencies,
    },
    program_publication_access,
};
#[test]
fn complete_query_sources_keep_unrelated_changes_and_reject_non_discovery_and_empty_leaf_changes() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = world::install(Graph::three_hops(false, vec![]));
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let result = read::run(&world, Forward::reference(), 0);
    assert_eq!(keys(&result), [0, 1, 2, 3]);
    let mut result = result.into_admitted_disclosed().into_output_demand_source();
    let retain = |result: &WorthQueryApplicationOutputDemandSource<Forward, NodeRow>| {
        world
            .application
            .with_host_advancement(|phase| {
                let custody = phase.test_host_request();
                let selected = world
                    .application
                    .select_product_branch(
                        &custody.borrowed_request_for_test(),
                        world.application.product_runtime().default_branch(),
                    )
                    .unwrap();
                selected
                    .retain_query_dependencies(&phase, &program_publication_access(), result)
                    .unwrap()
            })
            .unwrap()
    };
    let current = |dependencies: &WorthQueryRetainedQueryDependencies| {
        world
            .application
            .with_host_advancement(|phase| {
                let custody = phase.test_host_request();
                let selected = world
                    .application
                    .select_product_branch(
                        &custody.borrowed_request_for_test(),
                        world.application.product_runtime().default_branch(),
                    )
                    .unwrap();
                selected.require_current_required_set(
                    &phase,
                    &program_publication_access(),
                    &request,
                    std::iter::empty(),
                    dependencies,
                )
            })
            .unwrap()
    };
    let mut dependencies = retain(&result);
    let unrelated = program::set_node_active(&world, &request, 5, true);
    assert!(matches!(
        commit(&world, &request, unrelated, 240),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    current(&dependencies).unwrap();
    // Keep the original exact capture across the unrelated and hostile edit;
    // semantic currentness does not authorize re-capturing an old occurrence.
    for (target, key) in [(1, 241), (4, 242)] {
        let edge = program::link(&world, &request, 3, target);
        assert!(matches!(
            commit(&world, &request, edge, key),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        let refusal = current(&dependencies).unwrap_err();
        assert!(
            matches!(refusal,WorthQueryRequiredSetCurrentnessDenial::Output(ref cause) if cause.kind()==WorthQueryOutputDemandDenialKind::Superseded),
            "{refusal:?}"
        );
        let fresh = read::run(&world, Forward::reference(), 0);
        if target == 1 {
            assert_eq!(
                keys(&fresh),
                {
                    let mut values = result.rows().iter().map(|row| row.key).collect::<Vec<_>>();
                    values.sort();
                    values
                },
                "a nondiscovery cycle changes truth while preserving rows"
            );
        } else {
            assert_eq!(
                keys(&fresh),
                [0, 1, 2, 3, 4],
                "the old empty deep leaf was part of the source"
            );
        }
        assert_ne!(
            result.observed_sources()[0].idempotency_identity(),
            fresh.observed_sources()[0].idempotency_identity()
        );
        result = fresh.into_admitted_disclosed().into_output_demand_source();
        dependencies = retain(&result);
    }
}
