use crate::data::graph::SignalGraph;

use super::{
    InstalledSignalAuthorizationPolicy, SignalAuthorizationClauseContract,
    SignalAuthorizationClauseObservation, SignalAuthorizationDecision,
    SignalAuthorizationDependencyCardinality, SignalAuthorizationObservation,
    SignalAuthorizationPolicyDefinition, SignalAuthorizationPolicyIdentity,
    SignalAuthorizationRequirementContract, SignalAuthorizationRequirementObservation,
    SignalAuthorizationRuleContract, SignalAuthorizationRuleEffect,
    SignalAuthorizationRuleObservation,
};

#[test]
fn installed_nested_policy_preserves_or_and_prohibition_meaning() {
    let mut graph = SignalGraph::new();
    let graph_capability = graph.installed_graph_capability();
    let policy = graph
        .install_authorization_policy(&graph_capability, policy_definition([7; 32]))
        .unwrap();

    let primary_role = graph
        .evaluate_authorization(
            &policy,
            observation(&policy, [1; 32], [true, false], true, false),
        )
        .unwrap();
    let alternative_role = graph
        .evaluate_authorization(
            &policy,
            observation(&policy, [2; 32], [false, true], true, false),
        )
        .unwrap();
    let missing_actor = graph
        .evaluate_authorization(
            &policy,
            observation(&policy, [3; 32], [true, false], false, false),
        )
        .unwrap();
    let prohibited = graph
        .evaluate_authorization(
            &policy,
            observation(&policy, [4; 32], [true, false], true, true),
        )
        .unwrap();

    assert_eq!(
        primary_role.decision(),
        SignalAuthorizationDecision::Allowed
    );
    assert_eq!(
        alternative_role.decision(),
        SignalAuthorizationDecision::Allowed
    );
    assert_eq!(
        missing_actor.decision(),
        SignalAuthorizationDecision::Denied
    );
    assert_eq!(prohibited.decision(), SignalAuthorizationDecision::Denied);
    assert!(policy.retains(&primary_role));
    assert_eq!(primary_role.dependency_identity(), &[1; 32]);
    assert_eq!(primary_role.counters().rules_evaluated, 2);
    assert_eq!(primary_role.counters().requirements_evaluated, 3);
    assert_eq!(primary_role.counters().clauses_evaluated, 4);
    assert_eq!(primary_role.counters().entities_depended_on, 8);
    assert_eq!(
        missing_actor
            .rule_decisions()
            .iter()
            .map(|decision| (decision.effect(), decision.matched()))
            .collect::<Vec<_>>(),
        vec![
            (SignalAuthorizationRuleEffect::Required, false),
            (SignalAuthorizationRuleEffect::Prohibited, false),
        ]
    );
    assert_eq!(
        prohibited
            .rule_decisions()
            .iter()
            .map(|decision| (decision.effect(), decision.matched()))
            .collect::<Vec<_>>(),
        vec![
            (SignalAuthorizationRuleEffect::Required, true),
            (SignalAuthorizationRuleEffect::Prohibited, true),
        ]
    );
}

#[test]
fn nested_policy_rejects_shape_drift_and_non_exhaustive_clauses() {
    let mut graph = SignalGraph::new();
    let graph_capability = graph.installed_graph_capability();
    let policy = graph
        .install_authorization_policy(&graph_capability, policy_definition([9; 32]))
        .unwrap();

    let wrong_shape = SignalAuthorizationObservation::new(
        [5; 32],
        [rule_observation(
            SignalAuthorizationRuleEffect::Required,
            [requirement_observation([clause(&policy, true, true)])],
        )],
    );
    assert!(graph.evaluate_authorization(&policy, wrong_shape).is_err());

    let non_exhaustive = observation_with_clause(
        [6; 32],
        clause(&policy, true, false),
        clause(&policy, false, true),
        clause(&policy, true, true),
        clause(&policy, false, true),
    );
    assert!(graph
        .evaluate_authorization(&policy, non_exhaustive)
        .is_err());
}

fn policy_definition(identity: [u8; 32]) -> SignalAuthorizationPolicyDefinition {
    SignalAuthorizationPolicyDefinition::new(
        SignalAuthorizationPolicyIdentity::new(identity),
        [
            SignalAuthorizationRuleContract::all(
                SignalAuthorizationRuleEffect::Required,
                [
                    SignalAuthorizationRequirementContract::any([
                        SignalAuthorizationClauseContract::new(),
                        SignalAuthorizationClauseContract::new(),
                    ]),
                    SignalAuthorizationRequirementContract::any([
                        SignalAuthorizationClauseContract::new(),
                    ]),
                ],
            ),
            SignalAuthorizationRuleContract::all(
                SignalAuthorizationRuleEffect::Prohibited,
                [SignalAuthorizationRequirementContract::any([
                    SignalAuthorizationClauseContract::new(),
                ])],
            ),
        ],
    )
}

fn observation(
    policy: &InstalledSignalAuthorizationPolicy,
    identity: [u8; 32],
    roles: [bool; 2],
    actor: bool,
    prohibited: bool,
) -> SignalAuthorizationObservation {
    observation_with_clause(
        identity,
        clause(policy, roles[0], true),
        clause(policy, roles[1], true),
        clause(policy, actor, true),
        clause(policy, prohibited, true),
    )
}

fn observation_with_clause(
    identity: [u8; 32],
    role: SignalAuthorizationClauseObservation,
    alternate_role: SignalAuthorizationClauseObservation,
    actor: SignalAuthorizationClauseObservation,
    prohibited: SignalAuthorizationClauseObservation,
) -> SignalAuthorizationObservation {
    SignalAuthorizationObservation::new(
        identity,
        [
            rule_observation(
                SignalAuthorizationRuleEffect::Required,
                [
                    requirement_observation([role, alternate_role]),
                    requirement_observation([actor]),
                ],
            ),
            rule_observation(
                SignalAuthorizationRuleEffect::Prohibited,
                [requirement_observation([prohibited])],
            ),
        ],
    )
}

fn rule_observation(
    effect: SignalAuthorizationRuleEffect,
    requirements: impl IntoIterator<Item = SignalAuthorizationRequirementObservation>,
) -> SignalAuthorizationRuleObservation {
    SignalAuthorizationRuleObservation::all(effect, requirements)
}

fn requirement_observation(
    clauses: impl IntoIterator<Item = SignalAuthorizationClauseObservation>,
) -> SignalAuthorizationRequirementObservation {
    SignalAuthorizationRequirementObservation::any(clauses)
}

fn clause(
    policy: &InstalledSignalAuthorizationPolicy,
    matched: bool,
    exhaustive: bool,
) -> SignalAuthorizationClauseObservation {
    policy.admit_clause_observation(
        matched,
        exhaustive,
        SignalAuthorizationDependencyCardinality {
            entities: 2,
            relations: 1,
            adjacency_lists: 1,
            fields: 1,
        },
    )
}

#[test]
fn observations_from_a_different_installation_cannot_supply_matching_clauses() {
    let mut graph = SignalGraph::new();
    let token = graph.installed_graph_capability();
    let installed = graph
        .install_authorization_policy(&token, policy_definition([3; 32]))
        .unwrap();
    let mut other = SignalGraph::new();
    let token = other.installed_graph_capability();
    let foreign = other
        .install_authorization_policy(&token, policy_definition([3; 32]))
        .unwrap();

    let observation = observation(&foreign, [5; 32], [true, false], true, false);
    assert!(matches!(
        graph.evaluate_authorization(&installed, observation),
        Err(super::SignalAuthorizationDenial::ObservationShapeMismatch)
    ));
}
