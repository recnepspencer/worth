//! Presence predicates are canonical values, not aspect revision aliases.
use super::{kind, number_u64};
use worth_foundational::facade::CanonicalBasisEntry;

pub(super) fn append(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    entity_kind: worth_relational::facade::identity::KindId,
    present: bool,
) {
    kind(entries, prefix, "aspect-presence");
    number_u64(
        entries,
        prefix,
        "entity-kind",
        u64::from(entity_kind.as_u32()),
    );
    number_u64(entries, prefix, "present", u64::from(present));
}
