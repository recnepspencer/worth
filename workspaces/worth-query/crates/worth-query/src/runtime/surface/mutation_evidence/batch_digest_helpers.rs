use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryContinuityMutationEvidence, WorthQueryExistingTruthAssertionEvidence,
    WorthQueryExistingTruthBindingEvidence, WorthQueryMutationEvidenceDigest,
    WorthQueryMutationTargetEvidence, WorthQueryNamingMutationEvidence,
    WorthQuerySymbolicAspectResolutionEvidence, WorthQuerySymbolicTargetReferenceEvidence,
};

pub(super) fn batch_target_digest(
    components: &[WorthQueryMutationTargetEvidence],
) -> Result<WorthQueryMutationEvidenceDigest, IdentityDerivationDenial> {
    let identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), "batch-target")
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("component"),
                components
                    .iter()
                    .map(target_component_identity)
                    .collect::<Vec<_>>()
                    .iter(),
            )
            .seal()?;
    Ok(WorthQueryMutationEvidenceDigest::aggregate(
        "batch-target",
        identity,
    )?)
}

pub(super) fn batch_existing_truth_binding_digest(
    bindings: &[Option<WorthQueryExistingTruthBindingEvidence>],
) -> Result<Option<WorthQueryMutationEvidenceDigest>, IdentityDerivationDenial> {
    let bindings = bindings
        .iter()
        .flatten()
        .map(|binding| binding.binding_digest().evidence_identity().clone())
        .collect::<Vec<_>>();
    if bindings.is_empty() {
        return Ok(None);
    }
    Ok(Some(aggregate_sequence_identity(
        "batch-existing-truth-binding",
        bindings,
    )?))
}

pub(super) fn batch_existing_truth_assertion_digest(
    assertions: &[Option<WorthQueryExistingTruthAssertionEvidence>],
) -> Result<Option<WorthQueryMutationEvidenceDigest>, IdentityDerivationDenial> {
    let assertions = assertions
        .iter()
        .flatten()
        .map(existing_truth_assertion_identity)
        .collect::<Result<Vec<_>, _>>()?;
    if assertions.is_empty() {
        return Ok(None);
    }
    Ok(Some(aggregate_sequence_identity(
        "batch-existing-truth-assertion",
        assertions,
    )?))
}

pub(super) fn batch_symbolic_target_reference_digest(
    references: &[Option<WorthQuerySymbolicTargetReferenceEvidence>],
) -> Result<Option<WorthQueryMutationEvidenceDigest>, IdentityDerivationDenial> {
    let references = references
        .iter()
        .flatten()
        .map(|reference| symbolic_target_reference_identity("symbolic-target-reference", reference))
        .collect::<Result<Vec<_>, _>>()?;
    if references.is_empty() {
        return Ok(None);
    }
    Ok(Some(aggregate_sequence_identity(
        "batch-symbolic-target-reference",
        references,
    )?))
}

pub(super) fn batch_symbolic_resolution_digest(
    target_references: &[Option<WorthQuerySymbolicTargetReferenceEvidence>],
    aspect_resolutions: &[Vec<WorthQuerySymbolicAspectResolutionEvidence>],
) -> Result<Option<WorthQueryMutationEvidenceDigest>, IdentityDerivationDenial> {
    let rows = target_references
        .iter()
        .flatten()
        .map(|reference| {
            symbolic_target_reference_identity("symbolic-resolution-target", reference)
        })
        .chain(
            aspect_resolutions
                .iter()
                .flatten()
                .map(symbolic_aspect_resolution_identity),
        )
        .collect::<Result<Vec<_>, _>>()?;
    if rows.is_empty() {
        return Ok(None);
    }
    Ok(Some(aggregate_sequence_identity(
        "batch-symbolic-resolution",
        rows,
    )?))
}

pub(super) fn batch_continuity_mutation_digest(
    continuities: &[Option<WorthQueryContinuityMutationEvidence>],
) -> Result<Option<WorthQueryMutationEvidenceDigest>, IdentityDerivationDenial> {
    let continuities = continuities
        .iter()
        .flatten()
        .map(continuity_mutation_identity)
        .collect::<Result<Vec<_>, _>>()?;
    if continuities.is_empty() {
        return Ok(None);
    }
    Ok(Some(aggregate_sequence_identity(
        "batch-continuity-mutation",
        continuities,
    )?))
}

pub(super) fn batch_naming_mutation_digest(
    namings: &[Option<WorthQueryNamingMutationEvidence>],
) -> Result<Option<WorthQueryMutationEvidenceDigest>, IdentityDerivationDenial> {
    let namings = namings
        .iter()
        .flatten()
        .map(naming_mutation_identity)
        .collect::<Result<Vec<_>, _>>()?;
    if namings.is_empty() {
        return Ok(None);
    }
    Ok(Some(aggregate_sequence_identity(
        "batch-naming-mutation",
        namings,
    )?))
}

fn existing_truth_assertion_identity(
    assertion: &WorthQueryExistingTruthAssertionEvidence,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "existing-truth-assertion",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("mode"),
                assertion.mode().as_str(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("asserted_aspect_count"),
                assertion.asserted_aspect_count(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("verification"),
                assertion.verification_evidence_identity(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("assumption_snapshot"),
                assertion.assumption_snapshot_evidence_digest(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("verified_precondition"),
                assertion.verified_precondition_evidence_digest(),
            )
            .seal()?,
    )
}

fn symbolic_target_reference_identity(
    role: &'static str,
    reference: &WorthQuerySymbolicTargetReferenceEvidence,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                reference.family().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("symbol"),
                reference.symbol().evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("resolved_entity"),
                &reference.resolved_entity_identity().evidence_identity()?,
            );
    if let Some(collection) = reference.target_collection() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("target_collection"),
            collection.evidence_identity(),
        );
    }
    Ok(identity.seal()?)
}

fn symbolic_aspect_resolution_identity(
    resolution: &WorthQuerySymbolicAspectResolutionEvidence,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "symbolic-resolution-aspect",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                resolution.family().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("admitted_aspect_touch"),
                resolution.aspect_touch().admitted_touch_digest_part(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("symbol"),
                resolution.symbol().evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("resolved_entity"),
                &resolution.resolved_entity_identity().evidence_identity()?,
            );
    if let Some(collection) = resolution.target_collection() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("target_collection"),
            collection.evidence_identity(),
        );
    }
    Ok(identity.seal()?)
}

fn continuity_mutation_identity(
    continuity: &WorthQueryContinuityMutationEvidence,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), "continuity-mutation")
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                continuity.family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("outcome"),
                continuity.outcome_class().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("prior_authority"),
                continuity
                    .prior_authoritative_identity()
                    .evidence_identity(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("successor_authority"),
                continuity
                    .successor_authoritative_identities()
                    .iter()
                    .map(|identity| identity.evidence_identity()),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("lineage"),
                continuity.lineage_digest().evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("continuity_resolution"),
                continuity
                    .continuity_resolution_digest()
                    .evidence_identity(),
            );
    if let Some(digest) = continuity.basis_binding_digest() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("basis_binding"),
            digest.evidence_identity(),
        );
    }
    if let Some(resolved) = continuity.resolved_target_entity_identity() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("resolved_entity"),
            &resolved.evidence_identity()?,
        );
    }
    if let Some(collection) = continuity.target_collection() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("target_collection"),
            collection.evidence_identity(),
        );
    }
    Ok(identity.seal()?)
}

fn naming_mutation_identity(
    naming: &WorthQueryNamingMutationEvidence,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), "naming-mutation")
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                naming.family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("outcome"),
                naming.outcome().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("attachment"),
                naming.attachment_identity().evidence_identity(),
            );
    if let Some(prior) = naming.prior_authoritative_identity() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("prior_authority"),
            prior.evidence_identity(),
        );
    }
    if let Some(target) = naming.target_authoritative_identity() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("target_authority"),
            target.evidence_identity(),
        );
    }
    if let Some(resolved) = naming.resolved_target_entity_identity() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("resolved_entity"),
            &resolved.evidence_identity()?,
        );
    }
    if let Some(collection) = naming.target_collection() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("target_collection"),
            collection.evidence_identity(),
        );
    }
    Ok(identity.seal()?)
}

fn target_component_identity(
    component: &WorthQueryMutationTargetEvidence,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), "target-component")
            .field_shape(
                WorthQueryEvidenceTag::new("declared_class"),
                component.declared().target_class().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("resolved_class"),
                component.resolved().target_class().as_str(),
            );
    if let Some(collection) = component.declared().collection() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("declared_collection"),
            collection.evidence_identity(),
        );
    }
    if let Some(entity) = component.declared().entity_identity() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("declared_entity"),
            &entity.evidence_identity()?,
        );
    }
    if let Some(collection) = component.resolved().collection() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("resolved_collection"),
            collection.evidence_identity(),
        );
    }
    if let Some(entity) = component.resolved().entity_identity() {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("resolved_entity"),
            &entity.evidence_identity()?,
        );
    }
    Ok(identity.seal()?)
}

fn aggregate_sequence_identity(
    role: &'static str,
    entries: Vec<WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryMutationEvidenceDigest, IdentityDerivationDenial> {
    let identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("entry"), entries.iter())
            .seal()?;
    Ok(WorthQueryMutationEvidenceDigest::aggregate(role, identity)?)
}
