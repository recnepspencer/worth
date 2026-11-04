//! Independent full-compiler oracle for capability rule association.

use worth_runtime_bridge::facade::BridgeAuthorizationRuleEffect::Prohibited;
use worth_runtime_bridge::facade::BridgeAuthorizationRuleEffect::Required;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::ComposedCapabilityTouchOperation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::ComposedTouchAccountCapability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_composed_capability_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_seed::composition::CapabilityCompositionScenario;

#[test]
fn full_compiler_preserves_every_named_composition_rule_and_path() {
    let world = installed_composed_capability_world(CapabilityCompositionScenario::Lawful);
    let capability = world
        .application
        .installed_schema()
        .capability(
            ComposedTouchAccountCapability::reference(),
            ComposedCapabilityTouchOperation::reference(),
        )
        .expect("composed capability is installed");
    let plan = (*world
        .application
        .authorization_for_test())
        .capability_plan(&capability)
        .expect("the compiler retained the typed capability plan");

    // Derived independently from the declaration semantics: one grant path,
    // then allow, deny, conflict, separation-of-duty, and distinct-actor.
    let expected = [
        ("grant", Required, 0_usize, vec![vec![0_usize]]),
        ("allow", Required, 1, vec![vec![1]]),
        ("deny", Prohibited, 2, vec![vec![2]]),
        ("conflict", Prohibited, 3, vec![vec![3]]),
        ("separation-of-duty", Prohibited, 4, vec![vec![4]]),
        ("distinct-actor", Prohibited, 5, vec![vec![5]]),
    ];
    assert_eq!(plan.paths().len(), expected.len());
    assert_eq!(plan.rules().len(), expected.len());

    let decision_indices = [
        plan.decision_rules().grant,
        plan.decision_rules().allow,
        plan.decision_rules().deny.expect("deny rule"),
        plan.decision_rules().conflict.expect("conflict rule"),
        plan.decision_rules()
            .separation_of_duty
            .expect("separation-of-duty rule"),
        plan.decision_rules()
            .distinct_actor
            .expect("distinct-actor rule"),
    ];
    for ((name, effect, expected_index, expected_paths), actual_index) in
        expected.iter().zip(decision_indices)
    {
        assert_eq!(actual_index, *expected_index, "{name} index");
        let binding = &plan.rules()[actual_index];
        assert_eq!(binding.bridge().effect(), *effect, "{name} effect");
        assert_eq!(binding.path_requirements(), expected_paths, "{name} paths");
        let bridge_paths = binding
            .bridge()
            .requirements()
            .iter()
            .map(|requirement| {
                requirement
                    .clauses()
                    .iter()
                    .map(|clause| {
                        plan.paths()
                            .iter()
                            .position(|path| path.identity == *clause.identity())
                            .expect("Bridge clause names a compiled path")
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(bridge_paths, *expected_paths, "{name} Bridge association");
    }
}
