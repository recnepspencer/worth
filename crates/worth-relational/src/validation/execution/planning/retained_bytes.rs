use std::mem::size_of;

use crate::identity::data::KindId;
use crate::validation::data::{CustomInvariantAccessContract, InvariantRegistration};

pub(super) fn native_registration_retained_bytes(registration: &InvariantRegistration) -> u64 {
    crate::validation::execution::envelope::native_rule_bytes(&registration.rule)
}

fn kind_vec(kinds: &Vec<KindId>) -> u64 {
    (kinds.capacity() as u64).saturating_mul(size_of::<KindId>() as u64)
}

pub(super) fn custom_access_retained_bytes(access: &CustomInvariantAccessContract) -> u64 {
    kind_vec(&access.read_entity_kinds)
        .saturating_add(kind_vec(&access.read_relation_kinds))
        .saturating_add(kind_vec(&access.affected_entity_kinds))
        .saturating_add(kind_vec(&access.affected_relation_kinds))
}

pub(super) fn arc_bytes<T>() -> u64 {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .expect("planning layout fits")
        .0
        .pad_to_align()
        .size() as u64
}

pub(super) fn arc_slice_bytes<T>(count: usize) -> u64 {
    std::alloc::Layout::array::<T>(count)
        .and_then(|layout| std::alloc::Layout::new::<[usize; 2]>().extend(layout))
        .map(|(layout, _)| layout.pad_to_align().size() as u64)
        .unwrap_or(u64::MAX)
}
