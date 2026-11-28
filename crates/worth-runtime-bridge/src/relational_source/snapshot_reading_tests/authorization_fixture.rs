use crate::facade::{
    BridgeAuthorizationBindingIdentity, BridgeAuthorizationClauseContract as ClauseContract,
    BridgeAuthorizationClauseObservation as Clause,
    BridgeAuthorizationCorrespondenceIdentity as Correspondence, BridgeAuthorizationDenialKind,
    BridgeAuthorizationInstallationRequest, BridgeAuthorizationObservation,
    BridgeAuthorizationRequirementContract, BridgeAuthorizationRequirementObservation,
    BridgeAuthorizationRuleContract,
    BridgeAuthorizationRuleEffect::{self, Prohibited, Required},
    BridgeAuthorizationRuleObservation, BridgeAuthorizationRuntime,
};
use worth_foundational::facade::CanonicalDigestId;
use worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence;

pub(super) fn observed_rule(
    effect: BridgeAuthorizationRuleEffect,
    clause: Clause,
) -> BridgeAuthorizationRuleObservation {
    BridgeAuthorizationRuleObservation::all(
        effect,
        [BridgeAuthorizationRequirementObservation::any([clause])],
    )
}

pub(super) fn authorization(
    id: Correspondence,
    evidence: &RelationalAuthorizationObservationEvidence,
    clause: Clause,
) -> BridgeAuthorizationObservation {
    BridgeAuthorizationObservation::new(id, evidence, [observed_rule(Required, clause)])
}

pub(super) fn assert_shape_mismatch(
    bridge: &BridgeAuthorizationRuntime,
    observation: BridgeAuthorizationObservation,
) {
    let denial = bridge
        .evaluate(observation)
        .err()
        .expect("tampered clause must refuse");
    assert_eq!(
        denial.kind(),
        BridgeAuthorizationDenialKind::ObservationShapeMismatch
    );
}

pub(super) fn install_clause_policy(
    required: ClauseContract,
    prohibited: Option<ClauseContract>,
) -> (BridgeAuthorizationRuntime, Correspondence) {
    let mut bridge = BridgeAuthorizationRuntime::new();
    let digest = CanonicalDigestId::new([4; 32]);
    let rule = |effect, clause| {
        BridgeAuthorizationRuleContract::all(
            effect,
            [BridgeAuthorizationRequirementContract::any([clause])],
        )
    };
    let rules = [rule(Required, required)]
        .into_iter()
        .chain(prohibited.map(|clause| rule(Prohibited, clause)));
    let id = bridge
        .install(BridgeAuthorizationInstallationRequest::new(
            &digest,
            BridgeAuthorizationBindingIdentity::new(1, 1, digest.clone(), digest.clone()),
            "read",
            "subject",
            "self",
            rules,
        ))
        .unwrap();
    (bridge, id)
}
