use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;
use worth_relational::facade::runtime::{InvariantCatalog, InvariantRegistration, InvariantRule};

use super::super::test_support::{declaration_target, ready, ready_payload, success};
use super::super::{
    materialize_query_invariant_catalog_registration_artifact,
    WorthQueryInvariantCapabilityContributionAuthoring,
    WorthQueryInvariantCapabilityContributionPayload,
    WorthQueryInvariantCapabilityContributionPosture,
    WorthQueryInvariantRegistrationRuntimeSemantics,
};

#[test]
fn invariant_registration_materializer_builds_query_registration_artifact(
) -> Result<(), IdentityDerivationDenial> {
    let artifact = success(materialize_query_invariant_catalog_registration_artifact(
        ready_invariant_registration(
            WorthQueryInvariantCapabilityContributionAuthoring::invariant_rule_registration(
                InvariantRegistration::commit_boundary_blocking(InvariantRule::MaxMergedIntents(2)),
                "runtime.invariant.catalog_registration",
                "register merged-intent invariant at declaration scope",
            )?,
        )?,
    )?);

    assert_eq!(artifact.lane(), "query_invariant_catalog_registration");
    assert_eq!(
        artifact.semantic_code(),
        "runtime.invariant.catalog_registration"
    );
    assert_eq!(artifact.invariant_catalog().registrations.len(), 1);
    assert_eq!(
        artifact.invariant_catalog().registrations[0],
        InvariantRegistration::commit_boundary_blocking(InvariantRule::MaxMergedIntents(2))
    );
    assert_eq!(
        artifact.detail(),
        "register merged-intent invariant at declaration scope"
    );
    assert_eq!(artifact.intent_name(), "domain-capability.intent-invariant");
    assert_eq!(
        artifact.strategy_name(),
        "worth.domain_capability.intent-invariant"
    );

    Ok(())
}

#[test]
fn invariant_registration_materializer_denies_wrong_posture() -> Result<(), IdentityDerivationDenial>
{
    let wrong_posture = materialize_query_invariant_catalog_registration_artifact(
        ready_invariant_registration_payload(
            WorthQueryInvariantCapabilityContributionPayload::with_invariant_registration(
                WorthQueryInvariantCapabilityContributionPosture::SupportSummary,
                "runtime.invariant.catalog_registration",
                "registration semantics on the wrong posture should deny",
                Some(
                    WorthQueryInvariantRegistrationRuntimeSemantics::from_registration(
                        InvariantRegistration::commit_boundary_blocking(
                            InvariantRule::MaxMergedIntents(2),
                        ),
                    )?,
                ),
            )?,
        )?,
    )?;

    assert!(matches!(
        wrong_posture,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::super::WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture
    ));

    Ok(())
}

#[test]
fn invariant_registration_materializer_denies_missing_runtime_semantics(
) -> Result<(), IdentityDerivationDenial> {
    let missing =
        materialize_query_invariant_catalog_registration_artifact(ready_invariant_registration(
            WorthQueryInvariantCapabilityContributionAuthoring::support_summary(
                "runtime.invariant.catalog_registration",
                "registration semantics are absent here",
            )?,
        )?)?;

    assert!(matches!(
        missing,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::super::WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics
    ));

    Ok(())
}

#[test]
fn invariant_registration_materializer_accepts_empty_native_catalog(
) -> Result<(), IdentityDerivationDenial> {
    let artifact = success(materialize_query_invariant_catalog_registration_artifact(
        ready_invariant_registration(
            WorthQueryInvariantCapabilityContributionAuthoring::invariant_registration(
                InvariantCatalog {
                    registrations: vec![],
                },
                "runtime.invariant.catalog_registration",
                "empty catalog still carries explicit registration semantics",
            )?,
        )?,
    )?);

    assert!(artifact.invariant_catalog().registrations.is_empty());

    Ok(())
}

#[test]
fn invariant_registration_materializer_preserves_parity_across_catalog_ordering(
) -> Result<(), IdentityDerivationDenial> {
    let left = success(materialize_query_invariant_catalog_registration_artifact(
        ready_invariant_registration(
            WorthQueryInvariantCapabilityContributionAuthoring::invariant_registration(
                registration_catalog([
                    InvariantRegistration::commit_boundary_blocking(
                        InvariantRule::MaxMergedIntents(2),
                    ),
                    InvariantRegistration::snapshot_publication_blocking(
                        InvariantRule::MaxSnapshotEntities(4),
                    ),
                ]),
                "runtime.invariant.catalog_registration",
                "register two native invariants",
            )?,
        )?,
    )?);
    let right = success(materialize_query_invariant_catalog_registration_artifact(
        ready_invariant_registration(
            WorthQueryInvariantCapabilityContributionAuthoring::invariant_registration(
                registration_catalog([
                    InvariantRegistration::snapshot_publication_blocking(
                        InvariantRule::MaxSnapshotEntities(4),
                    ),
                    InvariantRegistration::commit_boundary_blocking(
                        InvariantRule::MaxMergedIntents(2),
                    ),
                ]),
                "runtime.invariant.catalog_registration",
                "register two native invariants",
            )?,
        )?,
    )?);
    let different = success(materialize_query_invariant_catalog_registration_artifact(
        ready_invariant_registration(
            WorthQueryInvariantCapabilityContributionAuthoring::invariant_registration(
                registration_catalog([
                    InvariantRegistration::commit_boundary_blocking(
                        InvariantRule::MaxMergedIntents(3),
                    ),
                    InvariantRegistration::snapshot_publication_blocking(
                        InvariantRule::MaxSnapshotEntities(4),
                    ),
                ]),
                "runtime.invariant.catalog_registration",
                "register two native invariants",
            )?,
        )?,
    )?);

    assert_eq!(
        left.materialization_digest(),
        right.materialization_digest()
    );
    assert_eq!(left, right);
    assert_ne!(
        left.materialization_digest(),
        different.materialization_digest()
    );
    assert_ne!(left, different);

    Ok(())
}

fn registration_catalog(
    registrations: impl IntoIterator<Item = InvariantRegistration>,
) -> InvariantCatalog {
    InvariantCatalog {
        registrations: registrations.into_iter().collect(),
    }
}

fn ready_invariant_registration(
    authoring: WorthQueryInvariantCapabilityContributionAuthoring,
) -> Result<
    super::super::WorthQueryMaterializationReadyInvariantCapabilityContribution<
        super::super::WorthQueryDeclarationBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    ready(authoring.bind_to_declaration_target(declaration_target("intent-invariant"))?)
}

fn ready_invariant_registration_payload(
    payload: WorthQueryInvariantCapabilityContributionPayload,
) -> Result<
    super::super::WorthQueryMaterializationReadyInvariantCapabilityContribution<
        super::super::WorthQueryDeclarationBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    ready_payload(declaration_target("intent-invariant"), payload)
}
