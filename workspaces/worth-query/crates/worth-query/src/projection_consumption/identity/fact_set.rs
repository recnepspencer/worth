mod family_chain;

#[cfg(test)]
mod tests;

use worth_foundational::facade::prepare_aspect_value_identity_basis;

use super::core::compose_extraction_counters_digest;
use super::scope::{scope_encoder, seal};
use crate::runtime::{WorthQueryContinuityMutationFamily, WorthQueryContinuityOutcomeClass};
use crate::WorthQueryEvidenceTag;
use family_chain::compose_consumed_projection_fact_family_digest;

use super::super::consumed::{
    ConsumedEffectContinuityFact, ConsumedEntityIdentityFact, ConsumedFieldValueFact,
    ConsumedMembershipFact, ConsumedRelationEndpointFact, ConsumedSourceReferenceFact,
    ConsumedTargetIdentityFact, ConsumedViewLocalIdentityFact, ProjectionFactExtractionCounters,
};
use super::super::contracts::ProjectionContractSupportPosture;
use super::super::eligibility::ProjectionConsumptionWarningKind;
use super::super::facts::ProjectionMaterializedFactPosture;
use super::super::source::ProjectionSourceFamily;

pub(crate) fn compose_consumed_projection_fact_set_digest(
    declaration_digest: &str,
    contract_digest: &str,
    source_family: ProjectionSourceFamily,
    source_identity: &str,
    support_posture: &ProjectionContractSupportPosture,
    materialized_fact_posture: Option<&ProjectionMaterializedFactPosture>,
    counters: &ProjectionFactExtractionCounters,
    entity_identities: &[ConsumedEntityIdentityFact],
    view_local_identities: &[ConsumedViewLocalIdentityFact],
    memberships: &[ConsumedMembershipFact],
    display_fields: &[ConsumedFieldValueFact],
    derived_fields: &[ConsumedFieldValueFact],
    target_identities: &[ConsumedTargetIdentityFact],
    source_references: &[ConsumedSourceReferenceFact],
    effect_continuity_facts: &[ConsumedEffectContinuityFact],
    relation_endpoints: &[ConsumedRelationEndpointFact],
) -> String {
    let mut encoder = scope_encoder("consumed_projection_fact_set_v2")
        .field_shape(
            WorthQueryEvidenceTag::new("declaration"),
            declaration_digest,
        )
        .field_shape(WorthQueryEvidenceTag::new("contract"), contract_digest)
        .field_shape(
            WorthQueryEvidenceTag::new("source_family"),
            source_family.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_identity"),
            source_identity,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("support_posture"),
            support_posture.as_str(),
        );
    if let Some(posture) = materialized_fact_posture {
        encoder = encoder.field_shape(
            WorthQueryEvidenceTag::new("materialized_fact_posture"),
            posture.posture_digest(),
        );
    }
    let entity_identity_digest = compose_consumed_projection_fact_family_digest(
        "entity_identity",
        entity_identities.iter().map(compose_entity_identity_entry),
    );
    let view_local_identity_digest = compose_consumed_projection_fact_family_digest(
        "view_local_identity",
        view_local_identities
            .iter()
            .map(compose_view_local_identity_entry),
    );
    let membership_digest = compose_consumed_projection_fact_family_digest(
        "membership",
        memberships.iter().map(compose_membership_entry),
    );
    let display_field_digest = compose_consumed_projection_fact_family_digest(
        "display_field",
        display_fields
            .iter()
            .map(|fact| compose_field_value_entry("display_field", fact)),
    );
    let derived_field_digest = compose_consumed_projection_fact_family_digest(
        "derived_field",
        derived_fields
            .iter()
            .map(|fact| compose_field_value_entry("derived_field", fact)),
    );
    let target_identity_digest = compose_consumed_projection_fact_family_digest(
        "target_identity",
        target_identities.iter().map(compose_target_identity_entry),
    );
    let source_reference_digest = compose_consumed_projection_fact_family_digest(
        "source_reference",
        source_references.iter().map(compose_source_reference_entry),
    );
    let effect_continuity_digest = compose_consumed_projection_fact_family_digest(
        "effect_continuity",
        effect_continuity_facts
            .iter()
            .flat_map(compose_effect_continuity_entries),
    );
    let relation_endpoint_digest = compose_consumed_projection_fact_family_digest(
        "relation_endpoint",
        relation_endpoints
            .iter()
            .map(compose_relation_endpoint_entry),
    );
    seal(
        encoder
            .field_value_sequence(
                WorthQueryEvidenceTag::new("warning_kind"),
                support_posture
                    .warning_kinds()
                    .iter()
                    .map(|warning: &ProjectionConsumptionWarningKind| warning.as_str()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("counters"),
                compose_extraction_counters_digest(counters),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("entity_identity_family"),
                entity_identity_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("view_local_identity_family"),
                view_local_identity_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("membership_family"),
                membership_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("display_field_family"),
                display_field_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("derived_field_family"),
                derived_field_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("target_identity_family"),
                target_identity_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("source_reference_family"),
                source_reference_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("effect_continuity_family"),
                effect_continuity_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("relation_endpoint_family"),
                relation_endpoint_digest,
            ),
    )
}

fn compose_entity_identity_entry(fact: &ConsumedEntityIdentityFact) -> String {
    let entity_identity = fact.entity_identity().evidence_identity();
    seal(
        scope_encoder("consumed_entity_identity_entry_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("source_row"),
                fact.source_row_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("entity_identity"),
                &entity_identity,
            ),
    )
}

fn compose_view_local_identity_entry(fact: &ConsumedViewLocalIdentityFact) -> String {
    seal(
        scope_encoder("consumed_view_local_identity_entry_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("source_row"),
                fact.source_row_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("view_local_identity"),
                fact.view_local_identity(),
            ),
    )
}

fn compose_membership_entry(fact: &ConsumedMembershipFact) -> String {
    seal(
        scope_encoder("consumed_membership_entry_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("source_row"),
                fact.source_row_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("grouping_aspect"),
                fact.native_grouping_aspect_key().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("grouping_value"),
                prepare_aspect_value_identity_basis(fact.grouping_value()).as_str(),
            ),
    )
}

fn compose_field_value_entry(family: &str, fact: &ConsumedFieldValueFact) -> String {
    let value_basis = fact.value_canonical_identity_basis();
    let encoder = scope_encoder("consumed_field_value_entry_v2")
        .field_shape(WorthQueryEvidenceTag::new("family"), family)
        .field_shape(
            WorthQueryEvidenceTag::new("source_row"),
            fact.source_row_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("field_key"),
            fact.field_path().terminal_projection_for_boundary(),
        );
    match value_basis {
        super::super::consumed::ConsumedNativeValueIdentityBasis::Value(value) => {
            seal(encoder.field_value(WorthQueryEvidenceTag::new("value"), value.as_str()))
        }
        super::super::consumed::ConsumedNativeValueIdentityBasis::Absent(posture) => {
            let posture = match posture {
                worth_foundational::facade::AbsenceLaw::Required => "required",
                worth_foundational::facade::AbsenceLaw::Optional => "optional",
                worth_foundational::facade::AbsenceLaw::Defaulted => "defaulted",
            };
            seal(encoder.field_shape(WorthQueryEvidenceTag::new("absence"), posture))
        }
    }
}

fn compose_target_identity_entry(fact: &ConsumedTargetIdentityFact) -> String {
    let target_identity = fact.target_identity().evidence_identity();
    seal(
        scope_encoder("consumed_target_identity_entry_v1").field_evidence_identity(
            WorthQueryEvidenceTag::new("target_identity"),
            &target_identity,
        ),
    )
}

fn compose_source_reference_entry(fact: &ConsumedSourceReferenceFact) -> String {
    seal(
        scope_encoder("consumed_source_reference_entry_v1")
            .field_shape(WorthQueryEvidenceTag::new("label"), fact.label())
            .field_shape(WorthQueryEvidenceTag::new("identity"), fact.identity()),
    )
}

fn compose_effect_continuity_entries(
    fact: &ConsumedEffectContinuityFact,
) -> impl Iterator<Item = String> + '_ {
    let primary = seal(
        scope_encoder("consumed_effect_continuity_entry_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                continuity_family_label(fact.family()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("outcome_class"),
                continuity_outcome_label(fact.outcome_class()),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("prior_authoritative_identity"),
                fact.prior_authoritative_identity().evidence_identity(),
            ),
    );
    let successors = fact
        .successor_authoritative_identities()
        .iter()
        .map(|identity| {
            seal(
                scope_encoder("consumed_effect_continuity_successor_entry_v1")
                    .field_evidence_identity(
                        WorthQueryEvidenceTag::new("successor_authoritative_identity"),
                        identity.evidence_identity(),
                    ),
            )
        });
    std::iter::once(primary).chain(successors)
}

fn compose_relation_endpoint_entry(fact: &ConsumedRelationEndpointFact) -> String {
    match fact {
        ConsumedRelationEndpointFact::MutationTarget {
            target_class,
            collection,
            entity_identity,
        } => {
            let mut encoder = scope_encoder("consumed_relation_endpoint_mutation_entry_v1")
                .field_shape(
                    WorthQueryEvidenceTag::new("target_class"),
                    target_class.as_str(),
                );
            if let Some(collection) = collection.as_deref() {
                encoder = encoder.field_shape(WorthQueryEvidenceTag::new("collection"), collection);
            }
            if let Some(entity_identity) = entity_identity {
                let evidence_identity = entity_identity.evidence_identity();
                encoder = encoder.field_evidence_identity(
                    WorthQueryEvidenceTag::new("entity_identity"),
                    &evidence_identity,
                );
            }
            seal(encoder)
        }
        ConsumedRelationEndpointFact::GroupedProjection {
            source_row_identity,
            grouping_aspect,
            grouping_value,
            ..
        } => seal(
            scope_encoder("consumed_relation_endpoint_grouped_entry_v1")
                .field_shape(
                    WorthQueryEvidenceTag::new("source_row"),
                    source_row_identity,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("grouping_aspect"),
                    grouping_aspect.as_str(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("grouping_value"),
                    prepare_aspect_value_identity_basis(grouping_value).as_str(),
                ),
        ),
    }
}

fn continuity_family_label(family: WorthQueryContinuityMutationFamily) -> &'static str {
    family.as_str()
}

fn continuity_outcome_label(outcome: WorthQueryContinuityOutcomeClass) -> &'static str {
    outcome.as_str()
}
