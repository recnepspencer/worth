use std::mem::size_of;

use crate::identity::data::KindId;
use crate::validation::data::{
    CustomInvariantAccessContract, InvariantRegistration, InvariantRule,
};

pub(super) fn native_registration_retained_bytes(registration: &InvariantRegistration) -> u64 {
    let nested = match &registration.rule {
        InvariantRule::UniqueEntityAspectField { field_locator } => {
            field_locator.owned_allocation_capacity_bytes() as u64
        }
        InvariantRule::EndpointKindContract(contract) => kind_vec(&contract.allowed_source_kinds)
            .saturating_add(kind_vec(&contract.allowed_target_kinds)),
        InvariantRule::CardinalityMinimumContract(contract) => {
            kind_vec(&contract.candidate_source_kinds)
                .saturating_add(kind_vec(&contract.candidate_target_kinds))
        }
        InvariantRule::ConnectivityMinimumContract(contract) => {
            kind_vec(&contract.source_kind_ids).saturating_add(kind_vec(&contract.target_kind_ids))
        }
        _ => 0,
    };
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
