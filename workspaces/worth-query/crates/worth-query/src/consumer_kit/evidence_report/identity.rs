use crate::evidence_identity::worth_query_evidence_identity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::field::{EvidenceReportField, EvidenceReportFieldValue};
use super::scope::EvidenceReportScope;

pub(crate) struct EvidenceReportIdentities {
    pub(crate) report_identity: WorthQueryEvidenceIdentity,
    pub(crate) field_inventory_identity: WorthQueryEvidenceIdentity,
    pub(crate) digest_participation_identity: WorthQueryEvidenceIdentity,
}

pub(crate) fn derive_evidence_report_identities(
    scope: &EvidenceReportScope,
    report_name: &str,
    fields: &[EvidenceReportField],
) -> Result<EvidenceReportIdentities, IdentityDerivationDenial> {
    let field_inventory_identities = derive_field_inventory_identities(fields)?;
    let participating_inventory_identities =
        select_participating_inventory_identities(fields, &field_inventory_identities);
    let field_inventory_identity = derive_report_field_inventory_identity(
        scope,
        report_name,
        fields.len(),
        &field_inventory_identities,
    )?;
    let digest_participation_identity = derive_report_digest_participation_identity(
        scope,
        report_name,
        &participating_inventory_identities,
    )?;
    let participating_value_identities = derive_participating_field_value_identities(fields)?;
    let report_identity = derive_report_value_identity(
        scope,
        report_name,
        &digest_participation_identity,
        &participating_value_identities,
    )?;

    Ok(EvidenceReportIdentities {
        report_identity,
        field_inventory_identity,
        digest_participation_identity,
    })
}

fn derive_field_inventory_identities(
    fields: &[EvidenceReportField],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    fields
        .iter()
        .map(derive_field_inventory_or_diagnostic_identity)
        .collect::<Result<Vec<_>, IdentityDerivationDenial>>()
}

fn select_participating_inventory_identities<'a>(
    fields: &'a [EvidenceReportField],
    field_inventory_identities: &'a [WorthQueryEvidenceIdentity],
) -> Vec<&'a WorthQueryEvidenceIdentity> {
    fields
        .iter()
        .zip(field_inventory_identities.iter())
        .filter_map(|(field, identity)| {
            field
                .participation()
                .participates_in_report_identity()
                .then_some(identity)
        })
        .collect()
}

fn derive_report_field_inventory_identity(
    scope: &EvidenceReportScope,
    report_name: &str,
    field_count: usize,
    field_inventory_identities: &[WorthQueryEvidenceIdentity],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(
            WorthQueryEvidenceScope::ConsumerEvidenceReportFieldInventory,
        )
        .field_shape(WorthQueryEvidenceTag::new("report_scope"), scope.as_str())
        .field_shape(WorthQueryEvidenceTag::new("report_name"), report_name)
        .field_usize(WorthQueryEvidenceTag::new("field_count"), field_count)
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("field_identity"),
            field_inventory_identities.iter(),
        )
        .seal()?,
    )
}

fn derive_report_digest_participation_identity(
    scope: &EvidenceReportScope,
    report_name: &str,
    participating_inventory_identities: &[&WorthQueryEvidenceIdentity],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(
        WorthQueryEvidenceScope::ConsumerEvidenceReportDigestParticipation,
    )
    .field_shape(WorthQueryEvidenceTag::new("report_scope"), scope.as_str())
    .field_shape(WorthQueryEvidenceTag::new("report_name"), report_name)
    .field_usize(
        WorthQueryEvidenceTag::new("participating_field_count"),
        participating_inventory_identities.len(),
    )
    .field_evidence_identity_sequence(
        WorthQueryEvidenceTag::new("participating_field_identity"),
        participating_inventory_identities.iter().copied(),
    )
    .seal()?)
}

fn derive_participating_field_value_identities(
    fields: &[EvidenceReportField],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    fields
        .iter()
        .filter(|field| field.participation().participates_in_report_identity())
        .map(derive_field_value_identity)
        .collect::<Result<Vec<_>, IdentityDerivationDenial>>()
}

fn derive_report_value_identity(
    scope: &EvidenceReportScope,
    report_name: &str,
    digest_participation_identity: &WorthQueryEvidenceIdentity,
    participating_value_identities: &[WorthQueryEvidenceIdentity],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerEvidenceReport)
            .field_shape(WorthQueryEvidenceTag::new("report_scope"), scope.as_str())
            .field_shape(WorthQueryEvidenceTag::new("report_name"), report_name)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("digest_participation_identity"),
                digest_participation_identity,
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("participating_field_identity"),
                participating_value_identities.iter(),
            )
            .seal()?,
    )
}

fn derive_field_inventory_identity(
    field: &EvidenceReportField,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerEvidenceReportField)
            .field_shape(WorthQueryEvidenceTag::new("field_name"), field.name())
            .field_shape(
                WorthQueryEvidenceTag::new("field_kind"),
                field.kind().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("participation"),
                field.participation().as_str(),
            )
            .seal()?,
    )
}

fn derive_field_inventory_or_diagnostic_identity(
    field: &EvidenceReportField,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(if field.participation().participates_in_report_identity() {
        derive_field_inventory_identity(field)?
    } else {
        derive_field_value_identity(field)?
    })
}

fn derive_field_value_identity(
    field: &EvidenceReportField,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let encoder =
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerEvidenceReportField)
            .field_shape(WorthQueryEvidenceTag::new("field_name"), field.name())
            .field_shape(
                WorthQueryEvidenceTag::new("field_kind"),
                field.kind().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("participation"),
                field.participation().as_str(),
            );

    Ok(match field.value() {
        EvidenceReportFieldValue::Shape(value) => encoder
            .field_shape(WorthQueryEvidenceTag::new("field_value"), value)
            .seal()?,
        EvidenceReportFieldValue::Value(value) => encoder
            .field_value(WorthQueryEvidenceTag::new("field_value"), value)
            .seal()?,
        EvidenceReportFieldValue::Bool(value) => encoder
            .field_bool(WorthQueryEvidenceTag::new("field_value"), *value)
            .seal()?,
        EvidenceReportFieldValue::Usize(value) => encoder
            .field_usize(WorthQueryEvidenceTag::new("field_value"), *value)
            .seal()?,
        EvidenceReportFieldValue::EvidenceIdentity(value) => encoder
            .field_evidence_identity(WorthQueryEvidenceTag::new("field_value"), value)
            .seal()?,
        EvidenceReportFieldValue::ValueSequence(values) => encoder
            .field_value_sequence(WorthQueryEvidenceTag::new("field_value"), values)
            .seal()?,
        EvidenceReportFieldValue::EvidenceIdentitySequence(values) => encoder
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("field_value"), values)
            .seal()?,
        EvidenceReportFieldValue::OptionalValue(value) => encoder
            .field_bool(WorthQueryEvidenceTag::new("field_present"), value.is_some())
            .optional_value(WorthQueryEvidenceTag::new("field_value"), value.as_deref())
            .seal()?,
        EvidenceReportFieldValue::OptionalEvidenceIdentity(value) => encoder
            .field_bool(WorthQueryEvidenceTag::new("field_present"), value.is_some())
            .optional_evidence_identity(WorthQueryEvidenceTag::new("field_value"), value.as_ref())
            .seal()?,
    })
}
