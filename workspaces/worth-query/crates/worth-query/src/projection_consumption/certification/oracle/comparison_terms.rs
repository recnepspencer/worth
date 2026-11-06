use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::projection_consumption::identity::{
    certification_scope_encoder, compose_certification_sequence_digest,
    compose_labeled_entry_digest, seal,
};
use crate::projection_consumption::ConsumedProjectionFactSet;
use crate::WorthQueryEvidenceTag;
use worth_foundational::facade::AspectKey;

use super::super::fixtures::{certification_grouped_projection, certification_row_set};
use super::value_terms::{
    canonical_aspect_value, canonical_consumed_native_value, canonical_snapshot_read_value,
    raw_string_snapshot_value,
};

pub(super) fn row_set_control_expected_digest(
    row_count: usize,
) -> Result<String, IdentityDerivationDenial> {
    let row_set = certification_row_set(row_count);
    let entries = row_set
        .rows()
        .iter()
        .flat_map(|row| {
            let entity_identity = row
                .projected_aspect_values()
                .iter()
                .find_map(|(field, value)| {
                    (field.as_str() == "identity.id")
                        .then(|| raw_string_snapshot_value(value))
                        .flatten()
                })
                .expect("identity.id should exist");
            let display_name = row
                .projected_aspect_values()
                .iter()
                .find_map(|(field, value)| {
                    (field.as_str() == "profile.display_name")
                        .then(|| canonical_snapshot_read_value(value))
                })
                .expect("display name should exist");
            [
                compose_oracle_entity_identity_entry(
                    row.row_identity().as_str(),
                    &crate::memory_workspace::admit_authored_entity_label(entity_identity),
                ),
                compose_oracle_display_field_entry(
                    row.row_identity().as_str(),
                    "profile.display_name",
                    &display_name,
                ),
            ]
        })
        .collect::<Vec<_>>();
    Ok(compose_certification_sequence_digest(
        "projection_consumption_oracle_row_set_control_expected_v1",
        "entry",
        entries,
    )?)
}

pub(super) fn row_set_control_actual_digest(
    facts: &ConsumedProjectionFactSet,
) -> Result<String, IdentityDerivationDenial> {
    let entries = facts
        .entity_identities()
        .iter()
        .map(|fact| {
            compose_oracle_entity_identity_entry(fact.source_row_identity(), fact.entity_identity())
        })
        .chain(facts.display_fields().iter().map(|fact| {
            compose_oracle_display_field_entry(
                fact.source_row_identity(),
                fact.field_path().terminal_projection_for_boundary(),
                &canonical_consumed_native_value(fact.native_value()),
            )
        }))
        .collect::<Vec<_>>();
    Ok(compose_certification_sequence_digest(
        "projection_consumption_oracle_row_set_control_actual_v1",
        "entry",
        entries,
    )?)
}

pub(super) fn grouped_worth_expected_digest(
    row_count: usize,
) -> Result<String, IdentityDerivationDenial> {
    let grouped = certification_grouped_projection(row_count);
    let entries = grouped
        .members()
        .iter()
        .flat_map(|member| {
            [
                compose_oracle_membership_entry(
                    member.row_identity().as_str(),
                    grouped.contract().grouping_aspect(),
                    &canonical_aspect_value(member.grouping_value()),
                ),
                compose_oracle_relation_endpoint_entry(
                    member.row_identity().as_str(),
                    Some(grouped.contract().grouping_aspect()),
                    &canonical_aspect_value(member.grouping_value()),
                ),
                compose_oracle_view_local_identity_entry(
                    member.row_identity().as_str(),
                    member.row_identity().as_str(),
                ),
            ]
        })
        .collect::<Vec<_>>();
    Ok(compose_certification_sequence_digest(
        "projection_consumption_oracle_grouped_worth_expected_v1",
        "entry",
        entries,
    )?)
}

pub(super) fn grouped_worth_actual_digest(
    facts: &ConsumedProjectionFactSet,
) -> Result<String, IdentityDerivationDenial> {
    let entries = facts
        .memberships()
        .iter()
        .map(|fact| {
            compose_oracle_membership_entry(
                fact.source_row_identity(),
                fact.native_grouping_aspect_key(),
                &canonical_aspect_value(fact.grouping_value()),
            )
        })
        .chain(facts.relation_endpoints().iter().map(|fact| {
            let grouping_value = fact
                .grouping_value()
                .map(canonical_aspect_value)
                .unwrap_or_else(|| "none".to_string());
            compose_oracle_relation_endpoint_entry(
                fact.source_row_identity().unwrap_or("none"),
                fact.native_grouping_aspect_key(),
                &grouping_value,
            )
        }))
        .chain(facts.view_local_identities().iter().map(|fact| {
            compose_oracle_view_local_identity_entry(
                fact.source_row_identity(),
                fact.view_local_identity(),
            )
        }))
        .collect::<Vec<_>>();
    Ok(compose_certification_sequence_digest(
        "projection_consumption_oracle_grouped_worth_actual_v1",
        "entry",
        entries,
    )?)
}

fn compose_oracle_entity_identity_entry(
    source_row: &str,
    entity_identity: &WorthQueryEntityIdentity,
) -> Result<String, IdentityDerivationDenial> {
    let evidence_identity = entity_identity.evidence_identity()?;
    Ok(seal(
        certification_scope_encoder("projection_consumption_oracle_entity_identity_entry_v1")
            .field_shape(WorthQueryEvidenceTag::new("source_row"), source_row)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("entity_identity"),
                &evidence_identity,
            ),
    )?)
}

fn compose_oracle_display_field_entry(
    source_row: &str,
    field_key: &str,
    value: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_labeled_entry_digest(
        "projection_consumption_oracle_display_field_entry_v1",
        &[
            ("source_row", source_row),
            ("field_key", field_key),
            ("value", value),
        ],
    )?)
}

fn compose_oracle_membership_entry(
    source_row: &str,
    grouping_aspect: &AspectKey,
    grouping_value: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_labeled_entry_digest(
        "projection_consumption_oracle_membership_entry_v1",
        &[
            ("source_row", source_row),
            (
                "grouping_aspect",
                terminal_grouping_aspect_digest_part(grouping_aspect),
            ),
            ("grouping_value", grouping_value),
        ],
    )?)
}

fn compose_oracle_relation_endpoint_entry(
    source_row: &str,
    grouping_aspect: Option<&AspectKey>,
    grouping_value: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_labeled_entry_digest(
        "projection_consumption_oracle_relation_endpoint_entry_v1",
        &[
            ("source_row", source_row),
            (
                "grouping_aspect",
                grouping_aspect
                    .map(terminal_grouping_aspect_digest_part)
                    .unwrap_or("none"),
            ),
            ("grouping_value", grouping_value),
        ],
    )?)
}

fn terminal_grouping_aspect_digest_part(grouping_aspect: &AspectKey) -> &str {
    grouping_aspect.as_str()
}

fn compose_oracle_view_local_identity_entry(
    source_row: &str,
    view_local_identity: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_labeled_entry_digest(
        "projection_consumption_oracle_view_local_identity_entry_v1",
        &[
            ("source_row", source_row),
            ("view_local_identity", view_local_identity),
        ],
    )?)
}
