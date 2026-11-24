use std::mem::size_of;

use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::validation::data::catalog::InvariantRegistration;
use crate::contracts::validation::data::descriptor::CustomInvariantAccessContract;

pub(super) fn native_registration_retained_bytes(registration: &InvariantRegistration) -> u64 {
    let nested = registration.rule.owned_allocation_capacity_bytes();
    nested.saturating_add(size_of::<InvariantRegistration>() as u64)
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
