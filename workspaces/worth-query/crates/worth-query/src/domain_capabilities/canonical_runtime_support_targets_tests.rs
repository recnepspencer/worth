use super::test_support::{declaration_target, lower_runtime_target, ready, success};
use super::{
    materialize_intent_declaration_support_traceability_artifact,
    materialize_lower_runtime_support_traceability_artifact,
    WorthQuerySupportContributionAuthoring, WorthQuerySupportContributionPayload,
    WorthQuerySupportContributionPosture,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[test]
fn declaration_support_traceability_materializer_builds_real_declaration_artifact(
) -> Result<(), IdentityDerivationDenial> {
    let artifact = success(
        materialize_intent_declaration_support_traceability_artifact(ready(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "intent.scope.traceability",
                "support stays attached to the authored declaration",
            )?
            .bind_to_declaration_target(declaration_target("intent-support"))
            .expect("fixture digest must fit"),
        )?)?,
    );

    assert_eq!(artifact.lane(), "domain_traceability");
    assert_eq!(
        artifact.support_detail(),
        "intent.scope.traceability:support stays attached to the authored declaration"
    );
    assert_eq!(artifact.intent_name(), "domain-capability.intent-support");
    assert_eq!(
        artifact.strategy_name(),
        "worth.domain_capability.intent-support"
    );
    assert_eq!(artifact.strategy_version(), "1");
    assert_eq!(artifact.input_contract(), "worth.domain-capability.fixture");
    assert_eq!(
        artifact.source_lane(),
        crate::runtime::WorthQueryIntentSourceLane::UserAuthored
    );
    assert_eq!(
        artifact.target_lane(),
        crate::runtime::WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert!(!artifact.target_binding_for_reporting().is_empty());
    assert!(!artifact.request_for_reporting().is_empty());
    assert!(!artifact.materialization_digest().is_empty());

    Ok(())
}

#[test]
fn lower_runtime_support_traceability_materializer_builds_real_boundary_artifact(
) -> Result<(), IdentityDerivationDenial> {
    let artifact = success(materialize_lower_runtime_support_traceability_artifact(
        ready(
            WorthQuerySupportContributionAuthoring::narrowed_support(
                "boundary.scope.support",
                "support stays attached to the lower runtime seam",
            )?
            .bind_to_lower_runtime_boundary_target(lower_runtime_target("boundary-support")?)
            .expect("fixture digest must fit"),
        )?,
    )?);

    assert_eq!(artifact.lane(), "domain_narrowed_support");
    assert_eq!(
        artifact.support_detail(),
        "boundary.scope.support:support stays attached to the lower runtime seam"
    );
    assert_eq!(
        artifact.seam_key(),
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting
    );
    assert_eq!(artifact.capability_label(), "Signal invalidation routing");
    assert_eq!(
        artifact.crossing_classification(),
        crate::lower_runtime_routing::WorthQueryLowerRuntimeCrossingClassification::QueryBoundaryAdapter
    );
    assert_eq!(
        artifact.route_kind(),
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteKind::RoutePlanning
    );
    assert_eq!(
        artifact.support_posture(),
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSupportPosture::Admitted
    );
    assert!(!artifact.envelope_for_reporting().is_empty());
    assert!(!artifact.target_binding_for_reporting().is_empty());
    assert!(!artifact.request_for_reporting().is_empty());
    assert!(!artifact.materialization_digest().is_empty());

    Ok(())
}

#[test]
fn declaration_support_traceability_digest_changes_when_scope_changes(
) -> Result<(), IdentityDerivationDenial> {
    let left = success(
        materialize_intent_declaration_support_traceability_artifact(ready(
            super::proof_integration::create_requested_domain_capability_contribution(
                declaration_target("intent-support-left"),
                WorthQuerySupportContributionPayload::new(
                    WorthQuerySupportContributionPosture::DeclarationSupport,
                    "intent.scope.support",
                    "support stays scoped to a declaration binding",
                )
                .expect("fixture digest must fit"),
            )?,
        )?)?,
    );
    let right = success(
        materialize_intent_declaration_support_traceability_artifact(ready(
            super::proof_integration::create_requested_domain_capability_contribution(
                declaration_target("intent-support-right"),
                WorthQuerySupportContributionPayload::new(
                    WorthQuerySupportContributionPosture::DeclarationSupport,
                    "intent.scope.support",
                    "support stays scoped to a declaration binding",
                )
                .expect("fixture digest must fit"),
            )?,
        )?)?,
    );

    assert_ne!(
        left.target_binding_for_reporting(),
        right.target_binding_for_reporting()
    );
    assert_ne!(
        left.materialization_digest(),
        right.materialization_digest()
    );

    Ok(())
}

#[test]
fn equivalent_declaration_support_meaning_materializes_same_artifact_digest(
) -> Result<(), IdentityDerivationDenial> {
    let authored = success(
        materialize_intent_declaration_support_traceability_artifact(ready(
            WorthQuerySupportContributionAuthoring::declaration_support(
                "intent.scope.support",
                "support stays scoped to a declaration binding",
            )?
            .bind_to_declaration_target(declaration_target("intent-support"))
            .expect("fixture digest must fit"),
        )?)?,
    );
    let direct = success(
        materialize_intent_declaration_support_traceability_artifact(ready(
            super::proof_integration::create_requested_domain_capability_contribution(
                declaration_target("intent-support"),
                WorthQuerySupportContributionPayload::new(
                    WorthQuerySupportContributionPosture::DeclarationSupport,
                    "intent.scope.support",
                    "support stays scoped to a declaration binding",
                )
                .expect("fixture digest must fit"),
            )?,
        )?)?,
    );

    assert_eq!(
        authored.materialization_digest(),
        direct.materialization_digest()
    );

    Ok(())
}

#[test]
fn lower_runtime_support_traceability_digest_changes_when_scope_changes(
) -> Result<(), IdentityDerivationDenial> {
    let left = success(materialize_lower_runtime_support_traceability_artifact(
        ready(
            super::proof_integration::create_requested_domain_capability_contribution(
                lower_runtime_target("boundary-support-left")?,
                WorthQuerySupportContributionPayload::new(
                    WorthQuerySupportContributionPosture::DeclarationTraceability,
                    "boundary.scope.traceability",
                    "support stays scoped to a lower-runtime binding",
                )
                .expect("fixture digest must fit"),
            )?,
        )?,
    )?);
    let right = success(materialize_lower_runtime_support_traceability_artifact(
        ready(
            super::proof_integration::create_requested_domain_capability_contribution(
                lower_runtime_target("boundary-support-right")?,
                WorthQuerySupportContributionPayload::new(
                    WorthQuerySupportContributionPosture::DeclarationTraceability,
                    "boundary.scope.traceability",
                    "support stays scoped to a lower-runtime binding",
                )
                .expect("fixture digest must fit"),
            )?,
        )?,
    )?);

    assert_ne!(
        left.target_binding_for_reporting(),
        right.target_binding_for_reporting()
    );
    assert_ne!(
        left.materialization_digest(),
        right.materialization_digest()
    );

    Ok(())
}

#[test]
fn equivalent_lower_runtime_support_meaning_materializes_same_artifact_digest(
) -> Result<(), IdentityDerivationDenial> {
    let authored = success(materialize_lower_runtime_support_traceability_artifact(
        ready(
            WorthQuerySupportContributionAuthoring::declaration_traceability(
                "boundary.scope.traceability",
                "support stays scoped to a lower-runtime binding",
            )?
            .bind_to_lower_runtime_boundary_target(lower_runtime_target("boundary-support")?)
            .expect("fixture digest must fit"),
        )?,
    )?);
    let direct = success(materialize_lower_runtime_support_traceability_artifact(
        ready(
            super::proof_integration::create_requested_domain_capability_contribution(
                lower_runtime_target("boundary-support")?,
                WorthQuerySupportContributionPayload::new(
                    WorthQuerySupportContributionPosture::DeclarationTraceability,
                    "boundary.scope.traceability",
                    "support stays scoped to a lower-runtime binding",
                )
                .expect("fixture digest must fit"),
            )?,
        )?,
    )?);

    assert_eq!(
        authored.materialization_digest(),
        direct.materialization_digest()
    );

    Ok(())
}
