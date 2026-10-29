pub(crate) use worth_query_worlds::facade::publication::application_authorization_publication::denial_taxonomy::*;

#[test]
fn real_missing_and_explicit_deny_publish_without_semantic_or_identity_aliasing() {
    let missing = real_denial(CompositionScenario::MissingAuthorization);
    let explicit = real_denial(CompositionScenario::ExplicitDeny);

    assert_eq!(missing.subject(), explicit.subject());
    assert_eq!(
        missing.kind(),
        Kind::Refused(AuthorizationRefusal::CapabilityAuthorizationMissing)
    );
    assert_eq!(
        explicit.kind(),
        Kind::Refused(AuthorizationRefusal::ExplicitDenyRuleMatched)
    );
    let published_missing = assert_publication(&missing, ExpectedDenialPublication::MISSING);
    let published_explicit = assert_publication(&explicit, ExpectedDenialPublication::EXPLICIT);

    assert_ne!(
        publication_identity(&published_missing),
        publication_identity(&published_explicit)
    );
}

#[test]
fn real_accumulated_prohibitions_retain_order_and_publish_explicit_deny_precedence() {
    let denial = real_denial(CompositionScenario::AccumulatedProhibitions);
    assert_eq!(
        denial.causes(),
        [
            Kind::Refused(AuthorizationRefusal::ExplicitDenyRuleMatched),
            Kind::Refused(AuthorizationRefusal::ConflictRuleMatched),
            Kind::Refused(AuthorizationRefusal::SeparationOfDutyRuleMatched),
            Kind::Refused(AuthorizationRefusal::DistinctActorRuleMatched),
        ]
    );
    assert_publication(&denial, ExpectedDenialPublication::EXPLICIT);
}
