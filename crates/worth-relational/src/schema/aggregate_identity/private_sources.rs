//! One compiled Native-private roster classifies law timing and aggregate inputs.
use crate::derived_relation::definition::RegisteredDerivedRelation;
use crate::identity::data::KindId;
use crate::schema::data::{AggregateIdentityContractDeclaration, RelationalSchemaRegistry};
use std::collections::BTreeSet;

pub(super) fn with_candidate(
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
) -> BTreeSet<KindId> {
    let mut private = registry.aggregate_private_kinds.clone();
    private.extend([
        contract.storage.aggregate_kind,
        contract.storage.lineage_kind,
        contract.membership_kind,
    ]);
    private
}

pub(crate) fn metadata_dependent(
    registry: &RelationalSchemaRegistry,
    definition: &RegisteredDerivedRelation,
) -> bool {
    definition.leaves.iter().any(|(relation, from, to)| {
        [relation, from, to]
            .into_iter()
            .any(|kind| registry.aggregate_private_kinds.contains(kind))
    }) || definition
        .carriers
        .iter()
        .any(|kind| registry.aggregate_private_kinds.contains(kind))
}
