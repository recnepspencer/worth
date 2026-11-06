use crate::basis_lifecycle::{
    NormalizedBasisIntent, ScopedMutationPreparationBasis, ScopedPreviewCloseoutBasis,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

pub(super) fn mutation_preparation_capability_identity(
    basis: &ScopedMutationPreparationBasis,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "effect_authoring_capability_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("variant"),
                "mutation_preparation",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                basis.family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authority"),
                basis.authority().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lifecycle"),
                basis.lifecycle().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("capability"),
                &basis_lifecycle_admitted_capability_label_identity(basis.capability_digest()),
            )
            .seal()?,
    )
}

pub(super) fn preview_closeout_capability_identity(
    basis: &ScopedPreviewCloseoutBasis,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "effect_authoring_capability_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("variant"), "preview_closeout")
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                basis.family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authority"),
                basis.authority().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lifecycle"),
                basis.lifecycle().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("capability"),
                &basis_lifecycle_admitted_capability_label_identity(basis.capability_digest()),
            )
            .seal()?,
    )
}

pub(super) fn advisory_basis_capability_identity(
    normalized: &NormalizedBasisIntent,
) -> WorthQueryEvidenceIdentity {
    capability_identity_for_normalized(normalized, "inspection_advisory")
}

pub(super) fn deferred_basis_capability_identity(
    normalized: &NormalizedBasisIntent,
) -> WorthQueryEvidenceIdentity {
    capability_identity_for_normalized(normalized, "deferred_future_neighbor")
}

fn capability_identity_for_normalized(
    normalized: &NormalizedBasisIntent,
    variant: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "effect_authoring_capability_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("variant"), variant)
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                normalized.family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authority"),
                normalized.authority().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lifecycle"),
                normalized.lifecycle().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("normalized"),
                &basis_lifecycle_normalized_label_identity(normalized),
            )
            .seal()?,
    )
}

pub(super) fn mutation_preparation_scoped_basis_identity(
    basis: &ScopedMutationPreparationBasis,
) -> WorthQueryEvidenceIdentity {
    scoped_basis_identity_for_admitted(
        "mutation_preparation",
        basis.family().as_str(),
        basis.authority().as_str(),
        basis.lifecycle().as_str(),
        basis_lifecycle_scoped_basis_label_identity(basis.scoped_basis_digest()),
    )
}

pub(super) fn preview_closeout_scoped_basis_identity(
    basis: &ScopedPreviewCloseoutBasis,
) -> WorthQueryEvidenceIdentity {
    scoped_basis_identity_for_admitted(
        "preview_closeout",
        basis.family().as_str(),
        basis.authority().as_str(),
        basis.lifecycle().as_str(),
        basis_lifecycle_scoped_basis_label_identity(basis.scoped_basis_digest()),
    )
}

fn scoped_basis_identity_for_admitted(
    variant: &str,
    family: &str,
    authority: &str,
    lifecycle: &str,
    scoped_basis: WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "effect_authoring_scoped_basis_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("variant"), variant)
            .field_shape(WorthQueryEvidenceTag::new("family"), family)
            .field_shape(WorthQueryEvidenceTag::new("authority"), authority)
            .field_shape(WorthQueryEvidenceTag::new("lifecycle"), lifecycle)
            .field_evidence_identity(WorthQueryEvidenceTag::new("scoped_basis"), &scoped_basis)
            .seal()?,
    )
}

pub(super) fn advisory_basis_scoped_basis_identity(
    normalized: &NormalizedBasisIntent,
) -> WorthQueryEvidenceIdentity {
    normalized_basis_identity(normalized, "inspection_advisory")
}

pub(super) fn deferred_basis_scoped_basis_identity(
    normalized: &NormalizedBasisIntent,
) -> WorthQueryEvidenceIdentity {
    normalized_basis_identity(normalized, "deferred_future_neighbor")
}

fn normalized_basis_identity(
    normalized: &NormalizedBasisIntent,
    variant: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "effect_authoring_scoped_basis_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("variant"), variant)
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                normalized.family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authority"),
                normalized.authority().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lifecycle"),
                normalized.lifecycle().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("normalized"),
                &basis_lifecycle_normalized_label_identity(normalized),
            )
            .seal()?,
    )
}

fn basis_lifecycle_admitted_capability_label_identity(
    capability_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "basis_lifecycle_admitted_capability_label_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("capability"), capability_digest)
            .seal()?,
    )
}

fn basis_lifecycle_scoped_basis_label_identity(
    scoped_basis_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "basis_lifecycle_scoped_basis_label_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("scoped_basis"),
                scoped_basis_digest,
            )
            .seal()?,
    )
}

fn basis_lifecycle_normalized_label_identity(
    normalized: &NormalizedBasisIntent,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "basis_lifecycle_normalized_label_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("normalized"),
                normalized.normalized_digest(),
            )
            .seal()?,
    )
}

pub(super) fn expected_lower_runtime_binding_identity(
    binding_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "expected_lower_runtime_binding_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("binding"), binding_digest)
            .seal()?,
    )
}
