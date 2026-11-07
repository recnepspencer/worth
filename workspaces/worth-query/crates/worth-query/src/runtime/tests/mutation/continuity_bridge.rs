use super::super::support::*;
use worth_runtime_bridge::facade::{
    BridgeContinuityAuthoritativeIdentity, BridgeContinuityMutationBundle,
    BridgeContinuityOutcomeClass, BridgeContinuityResolvedTargetIdentity,
    BridgeContinuityTargetCollection,
};

#[test]
fn bridge_split_successor_continuity_preserves_successor_set() {
    let evidence = WorthQueryContinuityMutationEvidence::from_bridge(
        &BridgeContinuityMutationBundle::split_existing_target(
            BridgeContinuityOutcomeClass::ContinuesAsSplitSuccessors,
            continuity_identity("authority:task-1"),
            [
                continuity_identity("authority:task-1:a"),
                continuity_identity("authority:task-1:b"),
            ],
            Some(resolved_target("entity:task-1")),
            Some(target_collection("Task")),
        )
        .expect("split continuity bundle should build"),
    )
    .expect("fixture digest must fit");

    assert_eq!(
        evidence.family(),
        WorthQueryContinuityMutationFamily::SplitExistingTarget
    );
    assert_eq!(
        evidence.outcome_class(),
        WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors
    );
    assert_eq!(
        evidence
            .successor_authoritative_identities()
            .iter()
            .map(|identity| identity.as_str())
            .collect::<Vec<_>>(),
        vec!["authority:task-1:a", "authority:task-1:b"]
    );
    assert_eq!(evidence.successor_authoritative_identity(), None);
    assert!(
        evidence.basis_binding_digest().is_none(),
        "direct bridge-only continuity evidence must not invent a query binding basis"
    );
}

#[test]
fn split_successor_intent_requires_at_least_two_successors() {
    let error = WorthQueryContinuityMutationIntent::split_existing_target(
        crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(
            crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:task-1")
                .expect("continuity prior authority label"),
        )
        .expect("continuity prior authority identity"),
        [
            crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
                crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(
                    "authority:task-1:a",
                )
                .expect("continuity successor authority label"),
            )
            .expect("continuity successor authority identity"),
        ],
    )
    .expect_err("split-successor continuity should reject a singleton successor set");

    assert_eq!(
        error.to_string(),
        "split-successor continuity requires at least two successor authoritative identities"
    );
}

fn continuity_identity(value: &str) -> BridgeContinuityAuthoritativeIdentity {
    BridgeContinuityAuthoritativeIdentity::new(value)
        .expect("test continuity identity should be native")
}

fn resolved_target(value: &str) -> BridgeContinuityResolvedTargetIdentity {
    BridgeContinuityResolvedTargetIdentity::new(value)
        .expect("test resolved target should be native")
}

fn target_collection(value: &str) -> BridgeContinuityTargetCollection {
    BridgeContinuityTargetCollection::new(value).expect("test target collection should be native")
}
