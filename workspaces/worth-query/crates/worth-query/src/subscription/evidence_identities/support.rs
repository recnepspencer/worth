use super::super::family::QuerySubscriptionFamily;
use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(in crate::subscription) fn subscription_family_capability_identity(
    family: &QuerySubscriptionFamily,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_family_capability_digest_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .seal()?,
    )
}

pub(in crate::subscription) fn support_subject_identity(
    support_class: &str,
    family: &QuerySubscriptionFamily,
    future_selection_identity: &WorthQueryEvidenceIdentity,
    declaration_identity: &WorthQueryEvidenceIdentity,
    admission_identity: Option<&WorthQueryEvidenceIdentity>,
    source_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut composer = WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_support_subject_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("support_class"), support_class)
        .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("future_selection"),
            future_selection_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("declaration"),
            declaration_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity);
    if let Some(admission_identity) = admission_identity {
        composer = composer
            .field_evidence_identity(WorthQueryEvidenceTag::new("admission"), admission_identity);
    }
    Ok(composer.seal()?)
}

pub(in crate::subscription) fn support_matrix_row_identity(
    family: &QuerySubscriptionFamily,
    support_class: &str,
    posture: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_support_matrix_row_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .field_shape(WorthQueryEvidenceTag::new("support_class"), support_class)
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture)
            .seal()?,
    )
}

pub(in crate::subscription) fn support_matrix_identity<'a>(
    family: &QuerySubscriptionFamily,
    capability_identity: &WorthQueryEvidenceIdentity,
    row_identities: impl IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_support_matrix_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("capability"),
                capability_identity,
            )
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), row_identities)
            .seal()?,
    )
}

pub(in crate::subscription) fn support_lookup_receipt_identity(
    family: &QuerySubscriptionFamily,
    support_class: &str,
    resolution_posture: &str,
    consumed_lookup_width: usize,
    remaining_lookup_width: usize,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_support_lookup_receipt_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .field_shape(WorthQueryEvidenceTag::new("support_class"), support_class)
            .field_shape(
                WorthQueryEvidenceTag::new("resolution_posture"),
                resolution_posture,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("consumed_lookup_width"),
                consumed_lookup_width,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("remaining_lookup_width"),
                remaining_lookup_width,
            )
            .seal()?,
    )
}

pub(in crate::subscription) fn support_counters_identity(
    support_report_request_count: u64,
    supported_family_count: u64,
    denied_family_count: u64,
    deferred_family_count: u64,
    uncertified_family_denial_count: u64,
    support_matrix_emission_count: u64,
    support_family_index_lookup_count: u64,
    support_matrix_scan_debt_count: u64,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_support_counters_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("support_report_request"),
                support_report_request_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("supported_family"),
                supported_family_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("denied_family"),
                denied_family_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("deferred_family"),
                deferred_family_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("uncertified_family_denial"),
                uncertified_family_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("support_matrix_emission"),
                support_matrix_emission_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("support_family_index_lookup"),
                support_family_index_lookup_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("support_matrix_scan_debt"),
                support_matrix_scan_debt_count as usize,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn support_report_identity(
    subject_identity: &WorthQueryEvidenceIdentity,
    posture: &str,
    matrix_identity: &WorthQueryEvidenceIdentity,
    lookup_receipt_identity: &WorthQueryEvidenceIdentity,
    counters_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_support_report_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("subject"), subject_identity)
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture)
            .field_evidence_identity(WorthQueryEvidenceTag::new("matrix"), matrix_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("lookup_receipt"),
                lookup_receipt_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity)
            .seal()?,
    )
}
