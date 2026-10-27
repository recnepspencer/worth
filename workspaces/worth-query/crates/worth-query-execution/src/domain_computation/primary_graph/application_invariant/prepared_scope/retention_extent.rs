//! Admission-set backing, separate from the native traversal's own allocations.
use super::{ApplicationInvariantAdmission, PlanningAdmission};
use worth_relational::facade::identity::{EntityId, RelationId};

pub(super) fn planning_arc_bytes() -> u64 {
    std::mem::size_of::<(usize, usize, std::sync::Mutex<PlanningAdmission>)>() as u64
}

pub(super) fn frozen_bytes(
    entities: usize,
    relations: usize,
    derived: usize,
    censuses: usize,
) -> u64 {
    (std::mem::size_of::<(usize, usize, ApplicationInvariantAdmission)>() as u64)
        .saturating_add((entities as u64).saturating_mul(entity_entry_bytes()))
        .saturating_add((relations as u64).saturating_mul(relation_entry_bytes()))
        .saturating_add((derived as u64).saturating_mul(derived_entry_bytes()))
        .saturating_add((censuses as u64).saturating_mul(census_entry_bytes()))
}

pub(super) fn census_entry_bytes() -> u64 {
    conservative_entry_bytes::<(
        worth_relational::facade::runtime::AggregateCensusReadIdentity,
        super::PreparedComponentCensus,
    )>()
}

pub(super) fn derived_entry_bytes() -> u64 {
    // Rust 1.94's insertion-only HashMap has at most four raw buckets per
    // admitted entry, including old/new tables while growing. Its control bytes
    // and group/alignment padding also fit this existing conservative bound of
    // eleven entry widths plus fourteen words per key. The frozen clone is
    // admitted separately before allocation, so both live owners remain paid.
    conservative_entry_bytes::<(
        worth_relational::facade::runtime::DerivedRelationReadIdentity,
        worth_relational::facade::runtime::PreparedDerivedRelationRead,
    )>()
}

pub(super) fn entity_entry_bytes() -> u64 {
    conservative_entry_bytes::<EntityId>()
}

pub(super) fn relation_entry_bytes() -> u64 {
    conservative_entry_bytes::<RelationId>()
}

fn conservative_entry_bytes<Key>() -> u64 {
    // Supported Rust 1.94: eleven keys, twelve child pointers, a parent pointer
    // and one word for length/index/padding. An insertion-only set has at most
    // one node per key. Quote before insertion and again before frozen copies
    // coexist, rather than treating native result backing as free admission.
    (11 * std::mem::size_of::<Key>() + 14 * std::mem::size_of::<usize>()) as u64
}
