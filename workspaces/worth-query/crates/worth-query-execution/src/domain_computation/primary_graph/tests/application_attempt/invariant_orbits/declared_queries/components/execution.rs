//! Independent Native component truth protects row grouping and typed inspection.
use super::*;
use worth_query_installation::facade::WorthQueryReadGraphPlanningContract;
#[test]
fn installed_components_keep_native_cohorts_and_original_result_carriers() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = interleaved_world();
    let installed = read::installed(&world, Census::reference());
    assert_eq!(
        WorthQueryReadGraphPlanningContract::component_selection(installed.read_graph()),
        Some(selection::<OrbitSchema<true>, Census>().meaning())
    );
    assert!(installed.read_graph().orbit().is_none());
    let result = read::run(&world, Census::reference(), 0);
    let native = native_oracle(&world);
    assert_eq!(native.iter().map(|(_, keys)| keys.len()).sum::<usize>(), 5);
    assert!(native.iter().any(|(_, keys)| keys
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        == [0, 2, 4].into_iter().collect()));
    assert!(native.iter().any(|(_, keys)| keys
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        == [1, 3].into_iter().collect()));
    let groups = result
        .components(selection::<OrbitSchema<true>, Census>())
        .unwrap();
    assert_eq!(groups.len(), 2);
    let actual = groups
        .iter()
        .map(|group| {
            (
                group.identity().clone(),
                group
                    .members()
                    .iter()
                    .map(|row| row.key)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, native);
    assert_eq!(
        result.rows().iter().map(|row| row.key).collect::<Vec<_>>(),
        native
            .iter()
            .flat_map(|(_, keys)| keys.iter().copied())
            .collect::<Vec<_>>()
    );
    let selected = read::run(&world, Of::reference(), 0);
    let of = selected.components(of::<OrbitSchema<true>>()).unwrap();
    assert_eq!(of.len(), 1);
    let steps = selected.receipt().root_path_steps();
    assert_eq!(
        steps.len(),
        1,
        "repeated member witnesses merge one authored step descriptor"
    );
    assert_eq!(steps[0].direction(), RelationDirection::Reverse);
    assert!(steps[0].reads().adjacency_lists_read() > 0);

    let one = of.iter().next().unwrap();
    let expected = native
        .iter()
        .find(|(_, members)| members.contains(&0))
        .unwrap();
    assert_eq!(one.identity(), &expected.0);
    assert_eq!(
        one.members().iter().map(|row| row.key).collect::<Vec<_>>(),
        expected.1
    );
    let admitted = result.into_admitted_disclosed();
    assert_eq!(
        admitted
            .components(selection::<OrbitSchema<true>, Census>())
            .unwrap()
            .len(),
        2
    );
    let demand = admitted.into_output_demand_source();
    assert_eq!(
        demand
            .components(selection::<OrbitSchema<true>, Census>())
            .unwrap()
            .iter()
            .map(|c| c.len())
            .sum::<usize>(),
        5
    );
    let metadata = read::run(&world, Components::reference(), 0);
    assert!(read::installed(&world, Components::reference())
        .read_graph()
        .projections()
        .is_empty());
    assert_eq!(metadata.rows().len(), 5);
    assert_eq!(
        metadata
            .components(selection::<OrbitSchema<true>, Components>())
            .unwrap()
            .len(),
        2
    );
}
fn native_oracle(
    world: &World,
) -> Vec<(
    WorthQueryAggregateIdentity<OrbitSchema<true>, Shells>,
    Vec<u64>,
)> {
    world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let mut groups = [0_u64, 1]
                            .map(|key| {
                                let root =
                                    reader.resolve_entity(NodeKey::reference(), key).unwrap();
                                let component =
                                    reader.component_of(Shells::reference(), &root).unwrap();
                                (
                                    component.identity().clone(),
                                    component
                                        .members()
                                        .map(|member| {
                                            reader.field(&member, NodeKey::reference()).unwrap()
                                        })
                                        .collect::<Vec<_>>(),
                                )
                            })
                            .to_vec();
                        groups.sort_by_key(|(identity, _)| *identity.bytes());
                        let by_entity = (0_u64..5).map(|key| {
                            let entity = reader.resolve_entity(NodeKey::reference(),key).unwrap();
                            (entity.entity_id(), key)
                        }).collect::<std::collections::BTreeMap<_,_>>();
                        assert_ne!(groups.iter().flat_map(|(_,keys)| keys.iter().copied()).collect::<Vec<_>>(), by_entity.into_values().collect::<Vec<_>>(), "the actual fixture must distinguish Native grouping from global entity sorting");

                        groups
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .unwrap()
}
#[test]
fn component_queries_refuse_partial_cohorts_and_limits_but_preserve_genuine_empty_census() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = interleaved_world();
    let host = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let denied = read::execute(&world, &execution, &request, Partial::reference(), 0)
        .err()
        .unwrap();
    assert!(
        matches!(denied,read::Stop::Read(ref stop) if stop.kind()==WorthQueryApplicationOneShotDenialKind::TraversalUnavailable),
        "{denied:?}"
    );
    let denied = read::execute_with_limit(&world, &execution, &request, Census::reference(), 0, 4)
        .err()
        .unwrap();
    assert!(
        matches!(denied,read::Stop::Read(ref stop) if stop.kind()==WorthQueryApplicationOneShotDenialKind::ResultLimitExceeded),
        "{denied:?}"
    );
    let empty = read::run(&world, Empty::reference(), 0);
    assert!(empty.rows().is_empty());
    assert!(empty
        .components(selection::<OrbitSchema<true>, Empty>())
        .unwrap()
        .is_empty());
    let body = super::custody::body(&world, empty.result_set_observation().source(), &request);
    assert!(body.aggregates().is_empty());
    assert!(body.operation_scope().is_none());
}
