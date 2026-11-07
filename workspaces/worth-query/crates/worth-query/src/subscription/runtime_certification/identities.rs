use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::coverage::{CoverageResolutionPosture, QuerySubscriptionFamilyCoverageRow};
use super::error::QuerySubscriptionRuntimeCertificationCounters;

#[allow(clippy::too_many_arguments)]
pub(super) fn coverage_row_identity(
    family: &str,
    row_class: &str,
    query_scope_identity: &WorthQueryEvidenceIdentity,
    subscription_family_identity: &WorthQueryEvidenceIdentity,
    subscription_declaration_identity: &WorthQueryEvidenceIdentity,
    bridge_declaration_identity: &WorthQueryEvidenceIdentity,
    signal_strategy_identity: &WorthQueryEvidenceIdentity,
    basis_identity: &WorthQueryEvidenceIdentity,
    policy_identity: &WorthQueryEvidenceIdentity,
    tenant_basis_identity: &WorthQueryEvidenceIdentity,
    relationship_proof_identity: &WorthQueryEvidenceIdentity,
    view_shape_identity: &WorthQueryEvidenceIdentity,
    support_report_identity: &WorthQueryEvidenceIdentity,
    bridge_parity_identity: &WorthQueryEvidenceIdentity,
    lifecycle_certification_identity: &WorthQueryEvidenceIdentity,
    diagnostic_bundle_identity: &WorthQueryEvidenceIdentity,
    lifecycle_class: &str,
    failure_identity: Option<&WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut composer =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_family_coverage_row_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family)
            .field_shape(WorthQueryEvidenceTag::new("row_class"), row_class)
            .field_evidence_identity(WorthQueryEvidenceTag::new("query"), query_scope_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("subscription_family"),
                subscription_family_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("subscription_declaration"),
                subscription_declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_declaration"),
                bridge_declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("signal_strategy"),
                signal_strategy_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("policy"), policy_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("tenant_basis"),
                tenant_basis_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("relationship_proof"),
                relationship_proof_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("view_shape"),
                view_shape_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("support_report"),
                support_report_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_parity"),
                bridge_parity_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("lifecycle_certification"),
                lifecycle_certification_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("diagnostic_bundle"),
                diagnostic_bundle_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lifecycle_class"),
                lifecycle_class,
            );
    if let Some(failure_identity) = failure_identity {
        composer = composer
            .field_evidence_identity(WorthQueryEvidenceTag::new("failure"), failure_identity);
    } else {
        composer = composer.field_shape(WorthQueryEvidenceTag::new("failure"), "none");
    }
    Ok(composer.seal()?)
}

pub(super) fn coverage_matrix_identity<'a, I>(
    rows: I,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>
where
    I: IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
{
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_family_coverage_matrix_v1",
            )
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("row"), rows)
            .seal()?,
    )
}

pub(super) fn coverage_evidence_variation_set_identity<'a, I>(
    identity_family: &str,
    values: I,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>
where
    I: IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
{
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                identity_family,
            )
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("variation"), values)
            .seal()?,
    )
}

pub(super) fn lifecycle_class_variation_set_identity<'a, I>(
    classes: I,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>
where
    I: IntoIterator<Item = &'a str>,
{
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_lifecycle_class_variation_set_v1",
            )
            .field_value_sequence(WorthQueryEvidenceTag::new("class"), classes)
            .seal()?,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn certified_family_coverage_handle_identity(
    family: &str,
    posture: &str,
    matrix_identity: &WorthQueryEvidenceIdentity,
    basis_variation_identity: &WorthQueryEvidenceIdentity,
    policy_variation_identity: &WorthQueryEvidenceIdentity,
    tenant_variation_identity: &WorthQueryEvidenceIdentity,
    relationship_proof_variation_identity: &WorthQueryEvidenceIdentity,
    view_shape_variation_identity: &WorthQueryEvidenceIdentity,
    lifecycle_class_variation_identity: &WorthQueryEvidenceIdentity,
    admitted_row_count: usize,
    hostile_row_count: usize,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_certified_family_coverage_handle_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family)
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture)
            .field_evidence_identity(WorthQueryEvidenceTag::new("matrix"), matrix_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis_variations"),
                basis_variation_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("policy_variations"),
                policy_variation_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("tenant_variations"),
                tenant_variation_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("relationship_proof_variations"),
                relationship_proof_variation_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("view_shape_variations"),
                view_shape_variation_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("lifecycle_class_variations"),
                lifecycle_class_variation_identity,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("admitted_row_count"),
                admitted_row_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("hostile_row_count"),
                hostile_row_count,
            )
            .seal()?,
    )
}

pub(super) fn coverage_width_identity(
    admitted_row_count: usize,
    hostile_row_count: usize,
    covered_variation_axis_count: usize,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_certification_coverage_width_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("admitted_rows"),
                admitted_row_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("hostile_rows"),
                hostile_row_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("covered_variation_axes"),
                covered_variation_axis_count,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(super) fn coverage_receipt_identity(
    coverage_resolution_posture: CoverageResolutionPosture,
    family_coverage_index_lookup_count: usize,
    covered_row_width: &WorthQueryEvidenceIdentity,
    uncovered_variation_width: usize,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_certification_coverage_receipt_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("coverage_resolution_posture"),
                coverage_resolution_posture.as_str(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("family_coverage_index_lookup_count"),
                family_coverage_index_lookup_count,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("covered_row_width"),
                covered_row_width,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("uncovered_variation_width"),
                uncovered_variation_width,
            )
            .seal()?,
    )
}

pub(super) fn hostile_coverage_identity(
    hostile_rows: &[QuerySubscriptionFamilyCoverageRow],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_runtime_hostile_coverage_v1",
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("hostile_row"),
                hostile_rows.iter().map(|row| row.row_identity()),
            )
            .seal()?,
    )
}

pub(super) fn runtime_certification_bundle_identity(
    scope_identity: &WorthQueryEvidenceIdentity,
    support_report_identity: &WorthQueryEvidenceIdentity,
    bridge_parity_identity: &WorthQueryEvidenceIdentity,
    diagnostic_bundle_identity: &WorthQueryEvidenceIdentity,
    lifecycle_certification_identity: &WorthQueryEvidenceIdentity,
    family_coverage_identity: &WorthQueryEvidenceIdentity,
    hostile_coverage_identity: &WorthQueryEvidenceIdentity,
    coverage_receipt_identity: &WorthQueryEvidenceIdentity,
    counter_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_runtime_certification_bundle_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("scope"), scope_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("support_report"),
                support_report_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_parity"),
                bridge_parity_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("diagnostic_bundle"),
                diagnostic_bundle_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("lifecycle_certification"),
                lifecycle_certification_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("family_coverage"),
                family_coverage_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("hostile_coverage"),
                hostile_coverage_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("coverage_receipt"),
                coverage_receipt_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counter_identity)
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(super) fn runtime_certification_counter_identity(
    counters: &QuerySubscriptionRuntimeCertificationCounters,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_runtime_certification_counters_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("certification_scope_emission_count"),
                counters.certification_scope_emission_count() as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("certified_family_count"),
                counters.certified_family_count() as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("hostile_row_coverage_count"),
                counters.hostile_row_coverage_count() as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("uncovered_family_denial_count"),
                counters.uncovered_family_denial_count() as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("family_coverage_index_lookup_count"),
                counters.family_coverage_index_lookup_count() as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("family_coverage_matrix_scan_debt_count"),
                counters.family_coverage_matrix_scan_debt_count() as usize,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}
