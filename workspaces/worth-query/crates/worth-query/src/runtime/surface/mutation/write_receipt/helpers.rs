use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::{WorthQueryEntityIdentity, WorthQuerySnapshotIdentity};
use crate::runtime::{
    WorthQueryContinuityMutationEvidence, WorthQueryContinuityMutationIntent,
    WorthQueryExistingTruthAssertionEvidence, WorthQueryExistingTruthAssertionMode,
    WorthQueryExistingTruthTargetBinding, WorthQueryMutationFamily, WorthQueryMutationTargetClass,
    WorthQueryMutationTargetCollectionIdentity, WorthQueryMutationTargetDescriptor,
    WorthQueryMutationTargetEvidence, WorthQueryNamingMutationEvidence,
    WorthQueryNamingMutationIntent, WorthQuerySymbolicTargetReference,
    WorthQuerySymbolicTargetReferenceEvidence, WorthQueryVerifiedExistingTruthAssertion,
};

pub(super) fn symbolic_target_reference_evidence(
    mutation_family: WorthQueryMutationFamily,
    bridge_reference: Option<&worth_runtime_bridge::facade::BridgeSymbolicTargetReferenceBundle>,
    authored_reference: Option<&WorthQuerySymbolicTargetReference>,
    resolved_entity_identity: Option<&WorthQueryEntityIdentity>,
) -> Result<Option<WorthQuerySymbolicTargetReferenceEvidence>, IdentityDerivationDenial> {
    if mutation_family == WorthQueryMutationFamily::Insert {
        return Ok(None);
    }
    Ok(bridge_reference
        .map(|bundle| {
            let target_collection = authored_reference.and_then(|reference| {
                reference
                    .target_collection()
                    .or_else(|| bundle.target_collection())
            });
            WorthQuerySymbolicTargetReferenceEvidence::from_bridge_with_query_context(
                bundle,
                resolved_entity_identity,
                target_collection,
            )
        })
        .or_else(|| {
            authored_reference.zip(resolved_entity_identity).map(
                |(reference, resolved_entity_identity)| {
                    WorthQuerySymbolicTargetReferenceEvidence::from_reference(
                        reference,
                        resolved_entity_identity,
                    )
                },
            )
        })?)
}

pub(super) fn naming_mutation_evidence(
    bridge_naming: Option<&worth_runtime_bridge::facade::BridgeNamingMutationBundle>,
    authored_intent: Option<&WorthQueryNamingMutationIntent>,
    resolved_target_entity_identity: Option<&WorthQueryEntityIdentity>,
    target_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
) -> Result<Option<WorthQueryNamingMutationEvidence>, IdentityDerivationDenial> {
    Ok(bridge_naming
        .map(|bundle| {
            WorthQueryNamingMutationEvidence::from_bridge_with_query_context(
                bundle,
                resolved_target_entity_identity,
                target_collection,
            )
        })
        .or_else(|| {
            authored_intent.map(|intent| {
                WorthQueryNamingMutationEvidence::from_intent(
                    intent,
                    resolved_target_entity_identity,
                    target_collection,
                )
            })
        })?)
}

pub(super) fn continuity_mutation_evidence(
    bridge_continuity: Option<&worth_runtime_bridge::facade::BridgeContinuityMutationBundle>,
    authored_intent: Option<&WorthQueryContinuityMutationIntent>,
    existing_truth_binding: Option<&crate::runtime::WorthQueryExistingTruthTargetBinding>,
    resolved_target_entity_identity: Option<&WorthQueryEntityIdentity>,
    target_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
) -> Result<Option<WorthQueryContinuityMutationEvidence>, IdentityDerivationDenial> {
    let basis_binding_identity =
        existing_truth_binding.map(|binding| binding.binding_evidence_identity());
    Ok(bridge_continuity
        .map(|bundle| {
            WorthQueryContinuityMutationEvidence::from_bridge_with_query_context(
                bundle,
                basis_binding_identity,
                resolved_target_entity_identity,
                target_collection,
            )
        })
        .or_else(|| {
            authored_intent.map(|intent| {
                WorthQueryContinuityMutationEvidence::from_intent(
                    intent,
                    basis_binding_identity,
                    resolved_target_entity_identity,
                    target_collection,
                )
            })
        })?)
}

pub(super) fn assertion_evidence(
    mutation_family: WorthQueryMutationFamily,
    existing_truth_binding: Option<&WorthQueryExistingTruthTargetBinding>,
    existing_truth_assertion: Option<&WorthQueryVerifiedExistingTruthAssertion>,
    declared_aspect_operations: &[crate::runtime::WorthQueryAspectMutationOperation],
    declared_aspect_value_digest: Option<&WorthQueryEvidenceIdentity>,
    snapshot_identity: &WorthQuerySnapshotIdentity,
) -> Result<Option<WorthQueryExistingTruthAssertionEvidence>, IdentityDerivationDenial> {
    let _binding = existing_truth_binding?;
    if let Some(verification) = existing_truth_assertion {
        return Ok(Some(
            WorthQueryExistingTruthAssertionEvidence::backend_verified(verification),
        ));
    }
    if mutation_family != WorthQueryMutationFamily::Assertion {
        return Ok(None);
    }
    Ok(Some(
        WorthQueryExistingTruthAssertionEvidence::retained_assertion(
            declared_aspect_operations.len(),
            retained_assertion_verification_digest(
                snapshot_identity,
                declared_aspect_value_digest,
            )?,
        ),
    ))
}

fn retained_assertion_verification_digest(
    snapshot_identity: &WorthQuerySnapshotIdentity,
    declared_aspect_value_digest: Option<&WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(
        WorthQueryEvidenceScope::RetainedExistingTruthAssertionEvidence,
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("snapshot_identity"),
        &snapshot_identity.evidence_identity(),
    )
    .optional_evidence_identity(
        WorthQueryEvidenceTag::new("declared_aspect_value_digest"),
        declared_aspect_value_digest,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("mode"),
        WorthQueryExistingTruthAssertionMode::RetainedAuthoritativeAssertion.as_str(),
    )
    .seal()?)
}

pub(super) fn target_evidence_from_receipt(
    mutation_family: WorthQueryMutationFamily,
    declared_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    declared_entity_identity: Option<WorthQueryEntityIdentity>,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    target_entity_identity: Option<WorthQueryEntityIdentity>,
) -> WorthQueryMutationTargetEvidence {
    let declared_target_class = match mutation_family {
        WorthQueryMutationFamily::Insert => WorthQueryMutationTargetClass::Collection,
        WorthQueryMutationFamily::Update
        | WorthQueryMutationFamily::Assertion
        | WorthQueryMutationFamily::Delete => WorthQueryMutationTargetClass::Entity,
    };
    let resolved_target_class = if target_entity_identity.is_some() {
        WorthQueryMutationTargetClass::Entity
    } else {
        WorthQueryMutationTargetClass::Collection
    };
    WorthQueryMutationTargetEvidence::new(
        WorthQueryMutationTargetDescriptor::new(
            declared_target_class,
            declared_collection,
            declared_entity_identity,
        ),
        WorthQueryMutationTargetDescriptor::new(
            resolved_target_class,
            target_collection,
            target_entity_identity,
        ),
    )
}
