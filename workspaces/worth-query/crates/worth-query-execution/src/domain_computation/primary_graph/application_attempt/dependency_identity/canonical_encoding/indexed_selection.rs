use worth_foundational::facade::{
    canonical_basis_value_for_aspect_value, CanonicalBasisEntry, CanonicalBasisEntryKind,
};

use super::{entities, kind, number, number_u64, push, text};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

pub(super) fn append(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    fact: &WorthQueryApplicationObservedFact,
) {
    let WorthQueryApplicationObservedFact::IndexedEntitySelection {
        index_id,
        definition,
        entity_kind,
        locator,
        value,
        candidate_limit,
        candidates,
        ..
    } = fact
    else {
        unreachable!("indexed selection encoder is called only for its fact variant");
    };
    kind(entries, prefix, "indexed-entity-selection");
    number_u64(entries, prefix, "index-id", index_id.0);
    push(
        entries,
        format!("{prefix}.definition-name"),
        CanonicalBasisEntryKind::Identity,
        text(definition.name.clone()),
    );
    number_u64(
        entries,
        prefix,
        "branch-scoped",
        u64::from(definition.branch_scoped),
    );
    push(
        entries,
        format!("{prefix}.aspect"),
        CanonicalBasisEntryKind::Locator,
        text(locator.aspect().aspect_key().as_str()),
    );
    for (index, field) in locator.field_path().fields().iter().enumerate() {
        push(
            entries,
            format!("{prefix}.field.{index}"),
            CanonicalBasisEntryKind::Locator,
            text(field.as_str()),
        );
    }
    number_u64(
        entries,
        prefix,
        "entity-kind",
        u64::from(entity_kind.as_u32()),
    );
    push(
        entries,
        format!("{prefix}.value"),
        CanonicalBasisEntryKind::Value,
        canonical_basis_value_for_aspect_value(value),
    );
    number(entries, prefix, "candidate-limit", *candidate_limit);
    entities(entries, prefix, "candidate", candidates);
}
