use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeCapabilityEligibility, WorthQueryLowerRuntimeCapabilityRequest,
    WorthQueryLowerRuntimeRouteSubjectIdentity, WorthQueryLowerRuntimeSubjectIdentity,
};

pub(super) fn fixture_retained_evidence_identity(
    fixture_family: impl AsRef<str>,
    retained_label: impl AsRef<str>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(WorthQueryEvidenceTag::new("fixture_family"), fixture_family)
        .field_value(
            WorthQueryEvidenceTag::new("fixture_retained_label"),
            retained_label,
        )
        .seal()
}

pub(super) fn fixture_subject_identity(
    subject_family: impl AsRef<str>,
    subject_label: impl AsRef<str>,
) -> Result<WorthQueryLowerRuntimeSubjectIdentity, IdentityDerivationDenial> {
    let evidence_identity =
        fixture_retained_evidence_identity(subject_family.as_ref(), subject_label)?;
    WorthQueryLowerRuntimeSubjectIdentity::compose(subject_family)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("fixture_subject"),
            &evidence_identity,
        )
        .seal()
}

pub(super) fn fixture_route_subject_identity(
    route_family: impl AsRef<str>,
    route_label: impl AsRef<str>,
) -> Result<WorthQueryLowerRuntimeRouteSubjectIdentity, IdentityDerivationDenial> {
    let evidence_identity = fixture_retained_evidence_identity(route_family.as_ref(), route_label)?;
    WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
        route_family,
        &evidence_identity,
    )
}

pub(super) fn admitted_fixture_eligibility(
    request: WorthQueryLowerRuntimeCapabilityRequest,
    detail_family: impl AsRef<str>,
    detail_label: impl AsRef<str>,
) -> Result<WorthQueryLowerRuntimeCapabilityEligibility, IdentityDerivationDenial> {
    let evidence_identity = fixture_retained_evidence_identity(detail_family, detail_label)?;
    admitted_fixture_eligibility_from_evidence(request, &evidence_identity)
}

pub(super) fn admitted_fixture_eligibility_from_evidence(
    request: WorthQueryLowerRuntimeCapabilityRequest,
    evidence_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryLowerRuntimeCapabilityEligibility, IdentityDerivationDenial> {
    WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request,
        evidence_identity,
    )
}
