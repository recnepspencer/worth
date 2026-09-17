use worth_foundational::facade::{
    canonical_basis_value_for_aspect_value, canonicalization, prepare_canonical_basis_sequence,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisValue, CanonicalDigestAlgorithmId, CanonicalDigestId, CanonicalDigestWorkBudget,
    CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;
use worth_relational::facade::identity::{EntityId, RelationId};

use super::super::WorthQueryApplicationObservedFact;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.application-producer-dependencies");
const RULE_VERSION: &str = "worth-query-application-producer-dependencies-v1";
const LINEAGE_RULE_VERSION: &str = "worth-query-application-producer-lineage-v1";
const MAXIMUM_CANONICAL_BYTES: usize = 4 * 1_024 * 1_024;

pub(super) fn dependency_identity(
    declared_key: [u8; 32],
    facts: &[WorthQueryApplicationObservedFact],
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), ()> {
    let mut entries = Vec::new();
    push(
        &mut entries,
        "declared-key".to_owned(),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(declared_key)),
    );
    push(
        &mut entries,
        "fact-count".to_owned(),
        CanonicalBasisEntryKind::Shape,
        unsigned(u64::try_from(facts.len()).unwrap_or(u64::MAX)),
    );
    for (index, fact) in facts.iter().enumerate() {
        let prefix = format!("fact.{index}");
        push(
            &mut entries,
            format!("{prefix}.locator"),
            CanonicalBasisEntryKind::Locator,
            text(fact.locator_identity()),
        );
        append_fact(&mut entries, &prefix, fact);
    }
    derive_identity(RULE_VERSION, entries)
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

fn derive_identity(
    rule_version: &str,
    entries: Vec<CanonicalBasisEntry>,
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), ()> {
    let version = CanonicalizationRuleVersion::new(rule_version).ok_or(())?;
    let maximum_entries = u32::try_from(entries.len()).map_err(|_| ())?;
    let budget =
        CanonicalDigestWorkBudget::new(maximum_entries, MAXIMUM_CANONICAL_BYTES).ok_or(())?;
    let basis = prepare_canonical_basis_sequence(version, DOMAIN, entries)
        .into_result()
        .map_err(|_| ())?;
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()
        .map_err(|_| ())?;
    let derived = canonicalization().digest().derive(ready);
    Ok((
        *derived.value().bytes(),
        WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    ))
}

fn append_fact(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    fact: &WorthQueryApplicationObservedFact,
) {
    match fact {
        WorthQueryApplicationObservedFact::SourceEntity { .. } => {
            kind(entries, prefix, "source-entity")
        }
        WorthQueryApplicationObservedFact::SourceAspectRevision {
            native_revision, ..
        } => {
            kind(entries, prefix, "source-aspect-revision");
            optional_u64(entries, prefix, "revision", *native_revision);
        }
        WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
            native_revision,
            comparison_work_limit,
            endpoints,
            ..
        } => {
            kind(entries, prefix, "source-adjacency-revision");
            optional_u64(
                entries,
                prefix,
                "revision",
                native_revision.map(|revision| revision.0),
            );
            number(
                entries,
                prefix,
                "comparison-work-limit",
                *comparison_work_limit,
            );
            entities(entries, prefix, "endpoint", endpoints);
        }
        WorthQueryApplicationObservedFact::Entity { .. } => kind(entries, prefix, "entity"),
        WorthQueryApplicationObservedFact::Field {
            kind: field_kind,
            value,
            ..
        } => {
            kind(entries, prefix, "field");
            number_u64(
                entries,
                prefix,
                "entity-kind",
                u64::from(field_kind.as_u32()),
            );
            push(
                entries,
                format!("{prefix}.value"),
                CanonicalBasisEntryKind::Value,
                canonical_basis_value_for_aspect_value(value),
            );
        }
        WorthQueryApplicationObservedFact::AbsentField {
            kind: field_kind, ..
        } => {
            kind(entries, prefix, "absent-field");
            number_u64(
                entries,
                prefix,
                "entity-kind",
                u64::from(field_kind.as_u32()),
            );
        }
        WorthQueryApplicationObservedFact::Relation {
            matching_relations, ..
        } => {
            kind(entries, prefix, "relation");
            relations(entries, prefix, "match", matching_relations);
        }
        WorthQueryApplicationObservedFact::Adjacency {
            maximum_work_units,
            relations: observed,
            ..
        } => {
            kind(entries, prefix, "adjacency");
            number(entries, prefix, "maximum-work-units", *maximum_work_units);
            number(entries, prefix, "relation-count", observed.len());
            for (index, relation) in observed.iter().enumerate() {
                record(
                    entries,
                    prefix,
                    &format!("relation.{index}.id"),
                    relation.relation_id,
                );
                entity(
                    entries,
                    prefix,
                    &format!("relation.{index}.from"),
                    relation.from,
                );
                entity(
                    entries,
                    prefix,
                    &format!("relation.{index}.to"),
                    relation.to,
                );
            }
        }
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
        value.map_or(CanonicalBasisValue::Null, |value| unsigned(value)),
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

fn unsigned(value: u64) -> CanonicalBasisValue {
    CanonicalBasisValue::UnsignedInteger {
        width: CanonicalIntegerWidth::Bits64,
        value: u128::from(value),
    }
}

fn text(value: impl Into<String>) -> CanonicalBasisValue {
    CanonicalBasisValue::ExactText(value.into().into())
}

fn push(
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
