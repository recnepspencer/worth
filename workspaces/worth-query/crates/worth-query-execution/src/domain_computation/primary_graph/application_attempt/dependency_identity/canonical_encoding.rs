use worth_foundational::facade::{
    canonical_basis_value_for_aspect_value, canonicalization, prepare_canonical_basis_sequence,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisValue, CanonicalDigestAlgorithmId, CanonicalDigestId, CanonicalIntegerWidth,
    CanonicalizationRuleVersion,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;
use worth_relational::facade::identity::{EntityId, RelationId};

use super::super::WorthQueryApplicationObservedFact;

mod aspect_presence;
mod fact_body;
use fact_body::append_fact;
pub(super) mod aggregate;
mod canonical_digest;
mod indexed_selection;
mod observation_identity;
mod source_publication;
pub(super) mod structural_walk;
pub(in crate::domain_computation::primary_graph::application_attempt) use observation_identity::authentication_observation_identity;
mod workflow_fact;
pub(super) use canonical_digest::derive_identity;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.application-producer-content-tree");
const RULE_VERSION: &str = "worth-query-application-producer-dependencies-v1";
const INDEX_GUARD_RULE_VERSION: &str =
    "worth-query-application-producer-dependencies-v2-index-definition";
const LINEAGE_RULE_VERSION: &str = "worth-query-application-producer-lineage-v2-content-tree";

/// One logical ordinal, independently of payload chunking or edit history.
pub(super) fn fact_entries(
    index: usize,
    fact: &WorthQueryApplicationObservedFact,
) -> Result<Vec<CanonicalBasisEntry>, ()> {
    if let WorthQueryApplicationObservedFact::IndexedEntitySelection { definition, .. } = fact {
        // This fact is native entity-field selection, not another index family.
        if !matches!(
            definition.kind,
            worth_relational::facade::indexes::DerivedIndexKind::EntityField { .. }
        ) {
            return Err(());
        }
    }
    let mut entries = Vec::new();
    let prefix = format!("fact.{index}");
    push(
        &mut entries,
        format!("{prefix}.locator"),
        CanonicalBasisEntryKind::Locator,
        text(fact.locator_identity()),
    );
    append_fact(&mut entries, &prefix, fact);
    Ok(entries)
}

pub(super) fn lineage_identity(
    dependency: [u8; 32],
    prior_head: Option<[u8; 32]>,
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), ()> {
    let mut entries = Vec::with_capacity(2);
    push(
        &mut entries,
        "dependency".to_owned(),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(dependency)),
    );
    push(
        &mut entries,
        "prior-lineage-head".to_owned(),
        CanonicalBasisEntryKind::Identity,
        prior_head.map_or(CanonicalBasisValue::Null, |identity| {
            CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(identity))
        }),
    );
    derive_identity(LINEAGE_RULE_VERSION, entries)
}

fn optional_entity(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    field: &str,
    value: Option<EntityId>,
) {
    match value {
        Some(value) => entity(entries, prefix, field, value),
        None => push(
            entries,
            format!("{prefix}.{field}"),
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::Null,
        ),
    }
}

fn kind(entries: &mut Vec<CanonicalBasisEntry>, prefix: &str, value: &str) {
    push(
        entries,
        format!("{prefix}.kind"),
        CanonicalBasisEntryKind::Shape,
        text(value),
    );
}

fn entities(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    field: &str,
    values: &[EntityId],
) {
    number(entries, prefix, &format!("{field}-count"), values.len());
    for (index, value) in values.iter().enumerate() {
        entity(entries, prefix, &format!("{field}.{index}"), *value);
    }
}

fn relations(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    field: &str,
    values: &[RelationId],
) {
    number(entries, prefix, &format!("{field}-count"), values.len());
    for (index, value) in values.iter().enumerate() {
        record(entries, prefix, &format!("{field}.{index}"), *value);
    }
}

fn entity(entries: &mut Vec<CanonicalBasisEntry>, prefix: &str, field: &str, value: EntityId) {
    push(
        entries,
        format!("{prefix}.{field}"),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::EntityRef {
            partition_id: value.partition_value(),
            local_slot: value.local_slot_value(),
            generation: value.generation_value(),
        },
    );
}

fn record(entries: &mut Vec<CanonicalBasisEntry>, prefix: &str, field: &str, value: RelationId) {
    push(
        entries,
        format!("{prefix}.{field}.partition"),
        CanonicalBasisEntryKind::Identity,
        unsigned(u64::from(value.partition_value())),
    );
    push(
        entries,
        format!("{prefix}.{field}.slot"),
        CanonicalBasisEntryKind::Identity,
        unsigned(value.local_slot_value()),
    );
    push(
        entries,
        format!("{prefix}.{field}.generation"),
        CanonicalBasisEntryKind::Identity,
        unsigned(u64::from(value.generation_value())),
    );
}

fn optional_u64(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    field: &str,
    value: Option<u64>,
) {
    push(
        entries,
        format!("{prefix}.{field}"),
        CanonicalBasisEntryKind::Value,
        value.map_or(CanonicalBasisValue::Null, unsigned),
    );
}

fn number(entries: &mut Vec<CanonicalBasisEntry>, prefix: &str, field: &str, value: usize) {
    number_u64(
        entries,
        prefix,
        field,
        u64::try_from(value).unwrap_or(u64::MAX),
    );
}

fn number_u64(entries: &mut Vec<CanonicalBasisEntry>, prefix: &str, field: &str, value: u64) {
    push(
        entries,
        format!("{prefix}.{field}"),
        CanonicalBasisEntryKind::Shape,
        unsigned(value),
    );
}

pub(super) fn unsigned(value: u64) -> CanonicalBasisValue {
    CanonicalBasisValue::UnsignedInteger {
        width: CanonicalIntegerWidth::Bits64,
        value: u128::from(value),
    }
}

fn text(value: impl Into<String>) -> CanonicalBasisValue {
    CanonicalBasisValue::ExactText(value.into().into())
}

pub(super) fn push(
    entries: &mut Vec<CanonicalBasisEntry>,
    locus: String,
    kind: CanonicalBasisEntryKind,
    value: CanonicalBasisValue,
) {
    entries.push(CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.into()),
        kind,
        value,
    ));
}
