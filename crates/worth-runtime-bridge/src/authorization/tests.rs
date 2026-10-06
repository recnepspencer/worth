use worth_foundational::facade::CanonicalDigestId;

use super::{
    BridgeAuthorizationBindingIdentity, BridgeAuthorizationClauseContract,
    BridgeAuthorizationClauseObservation, BridgeAuthorizationDependencyCardinality,
    BridgeAuthorizationInstallationBatch, BridgeAuthorizationInstallationRequest,
    BridgeAuthorizationObservation, BridgeAuthorizationRequirementContract,
    BridgeAuthorizationRequirementObservation, BridgeAuthorizationRetentionStop,
    BridgeAuthorizationRuleContract, BridgeAuthorizationRuleEffect,
    BridgeAuthorizationRuleObservation, BridgeAuthorizationRuntime,
};

#[test]
fn invalid_batch_cannot_install_its_valid_prefix() {
    let mut runtime = BridgeAuthorizationRuntime::new();
    let mut batch = BridgeAuthorizationInstallationBatch::new();
    batch
        .add(installation_request([21; 32], rules()))
        .expect("valid prefix");
    let denial = batch
        .add(installation_request([22; 32], Vec::new()))
        .expect_err("empty policy poisons the batch");
    assert_eq!(
        denial.kind(),
        super::BridgeAuthorizationDenialKind::EmptyPolicy
    );
    assert_eq!(
        runtime.install_batch(batch).unwrap_err().kind(),
        super::BridgeAuthorizationDenialKind::EmptyPolicy
    );
    runtime
        .install(installation_request([21; 32], rules()))
        .expect("failed batch installed no prefix");
}

#[test]
fn duplicate_inside_batch_cannot_install_the_first_request() {
    let mut runtime = BridgeAuthorizationRuntime::new();
    let mut batch = BridgeAuthorizationInstallationBatch::new();
    batch
        .add(installation_request([23; 32], rules()))
        .expect("first request");
    assert_eq!(
        batch
            .add(installation_request([23; 32], rules()))
            .unwrap_err()
            .kind(),
        super::BridgeAuthorizationDenialKind::DuplicateCorrespondence
    );
    assert_eq!(
        runtime.install_batch(batch).unwrap_err().kind(),
        super::BridgeAuthorizationDenialKind::DuplicateCorrespondence
    );
    runtime
        .install(installation_request([23; 32], rules()))
        .expect("duplicate batch installed no prefix");
}

#[test]
fn installed_correspondence_retains_nested_signal_decision_and_dependency_identity() {
    let mut runtime = BridgeAuthorizationRuntime::new();
    let rules = rules();
    let identity = runtime
        .install(BridgeAuthorizationInstallationRequest::new(
            &CanonicalDigestId::new([9; 32]),
            binding_identity(),
            "view_account",
            "Account",
            "account_membership",
            rules.clone(),
        ))
        .unwrap();
    let evidence = runtime
        .evaluate(observation(identity, &rules, [8; 32], true, false))
        .unwrap();

    assert!(runtime.retains(&evidence));
    assert_eq!(evidence.dependency_identity(), &[8; 32]);
    assert_eq!(
        evidence.decision(),
        worth_signal::facade::SignalAuthorizationDecision::Allowed
    );
    assert_eq!(evidence.counters().requirements_evaluated, 2);
    assert_eq!(
        evidence
            .rule_decisions()
            .iter()
            .map(|decision| (decision.effect(), decision.matched()))
            .collect::<Vec<_>>(),
        vec![
            (BridgeAuthorizationRuleEffect::Required, true),
            (BridgeAuthorizationRuleEffect::Prohibited, false),
        ]
    );
}

#[test]
fn admitted_retention_preserves_signal_decision_and_original_late_stop() {
    let mut runtime = BridgeAuthorizationRuntime::new();
    let rules = rules();
    let identity = runtime
        .install(installation_request([31; 32], rules.clone()))
        .unwrap();
    let evidence = runtime
        .evaluate(observation(identity, &rules, [13; 32], true, false))
        .unwrap();
    let mut measured = 0_u64;
    assert_eq!(
        runtime.retains_admitted(&evidence, |work, bytes| {
            assert_eq!(bytes, 0);
            measured += work;
            Ok::<_, &'static str>(())
        }),
        Ok(true)
    );
    assert!(runtime.retains(&evidence));
    assert!(measured > 3);
    let mut spent = 0_u64;
    assert!(matches!(
        runtime.retains_admitted(&evidence, |work, _| {
            if spent + work > measured - 1 {
                return Err("carried work exhausted");
            }
            spent += work;
            Ok(())
        }),
        Err(BridgeAuthorizationRetentionStop::Admission(
            "carried work exhausted"
        ))
    ));
    assert!(spent > 0);
}

#[test]
fn correspondence_rejects_clause_reordering_and_non_exhaustive_facts() {
    let mut runtime = BridgeAuthorizationRuntime::new();
    let rules = rules();
    let identity = runtime
        .install(BridgeAuthorizationInstallationRequest::new(
            &CanonicalDigestId::new([10; 32]),
            binding_identity(),
            "approve_payment",
            "PaymentIntent",
            "distinct_approver",
            rules.clone(),
        ))
        .unwrap();

    let mut reordered = observation(identity, &rules, [3; 32], true, false);
    reordered.rules[0].requirements[0].clauses.swap(0, 1);
    assert!(runtime.evaluate(reordered).is_err());

    let incomplete = observation(identity, &rules, [4; 32], true, false);
    let BridgeAuthorizationObservation {
        correspondence,
        dependency_identity,
        mut rules,
    } = incomplete;
    rules[0].requirements[0].clauses[0] =
        clause_observation(*rules[0].requirements[0].clauses[0].identity(), true, false);
    assert!(runtime
        .evaluate(BridgeAuthorizationObservation {
            correspondence,
            dependency_identity,
            rules,
        })
        .is_err());
}

fn rules() -> Vec<BridgeAuthorizationRuleContract> {
    vec![
        BridgeAuthorizationRuleContract::all(
            BridgeAuthorizationRuleEffect::Required,
            [BridgeAuthorizationRequirementContract::any([
                BridgeAuthorizationClauseContract::new([1; 32]),
                BridgeAuthorizationClauseContract::new([2; 32]),
            ])],
        ),
        BridgeAuthorizationRuleContract::all(
            BridgeAuthorizationRuleEffect::Prohibited,
            [BridgeAuthorizationRequirementContract::any([
                BridgeAuthorizationClauseContract::new([3; 32]),
            ])],
        ),
    ]
}

fn installation_request(
    identity: [u8; 32],
    rules: Vec<BridgeAuthorizationRuleContract>,
) -> BridgeAuthorizationInstallationRequest {
    BridgeAuthorizationInstallationRequest::new(
        &CanonicalDigestId::new(identity),
        binding_identity(),
        "batch_ability",
        "BatchScope",
        format!("batch-policy-{}", identity[0]),
        rules,
    )
}

fn observation(
    identity: super::BridgeAuthorizationCorrespondenceIdentity,
    rules: &[BridgeAuthorizationRuleContract],
    dependency_identity: [u8; 32],
    allowed: bool,
    prohibited: bool,
) -> BridgeAuthorizationObservation {
    BridgeAuthorizationObservation::new(
        identity,
        dependency_identity,
        [
            rule_observation(
                &rules[0],
                [requirement_observation(
                    &rules[0].requirements()[0],
                    [(allowed, true), (false, true)],
                )],
            ),
            rule_observation(
                &rules[1],
                [requirement_observation(
                    &rules[1].requirements()[0],
                    [(prohibited, true)],
                )],
            ),
        ],
    )
}

fn rule_observation(
    contract: &BridgeAuthorizationRuleContract,
    requirements: impl IntoIterator<Item = BridgeAuthorizationRequirementObservation>,
) -> BridgeAuthorizationRuleObservation {
    BridgeAuthorizationRuleObservation::all(contract.effect(), requirements)
}

fn requirement_observation<const N: usize>(
    contract: &BridgeAuthorizationRequirementContract,
    states: [(bool, bool); N],
) -> BridgeAuthorizationRequirementObservation {
    BridgeAuthorizationRequirementObservation::any(contract.clauses().iter().zip(states).map(
        |(clause, (matched, exhaustive))| {
            clause_observation(*clause.identity(), matched, exhaustive)
        },
    ))
}

fn clause_observation(
    identity: [u8; 32],
    matched: bool,
    exhaustive: bool,
) -> BridgeAuthorizationClauseObservation {
    BridgeAuthorizationClauseObservation::new(
        identity,
        matched,
        exhaustive,
        BridgeAuthorizationDependencyCardinality {
            entities: 2,
            relations: 1,
            adjacency_lists: 1,
            fields: 1,
        },
    )
}

fn binding_identity() -> BridgeAuthorizationBindingIdentity {
    BridgeAuthorizationBindingIdentity::new(
        1,
        1,
        CanonicalDigestId::new([11; 32]),
        CanonicalDigestId::new([12; 32]),
    )
}
