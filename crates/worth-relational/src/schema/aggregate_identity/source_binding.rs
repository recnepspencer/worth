//! Installed graph sources share member meaning, never private storage roles.
use crate::schema::data::{
    AggregateIdentityContractDeclaration, AggregateIdentitySource, MinimumCardinalityEnforcement,
    RelationalSchemaRegistry,
};
use std::collections::BTreeSet;

pub(super) fn valid(
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
) -> bool {
    let reserved = [
        contract.member_kind,
        contract.membership_kind,
        contract.storage.aggregate_kind,
        contract.storage.lineage_kind,
    ];
    if reserved.into_iter().collect::<BTreeSet<_>>().len() != reserved.len()
        || contract.source.relations().is_empty()
    {
        return false;
    }
    contract.source.relations().iter().all(|kind| {
        if reserved.contains(kind) {
            return false;
        }
        let Some(source) = registry.relation_kinds.get(kind) else {
            return false;
        };
        let homogeneous = source
            .relation_integrity
            .endpoint_kind_contracts
            .iter()
            .any(|endpoints| {
                endpoints.allowed_source_kinds == [contract.member_kind]
                    && endpoints.allowed_target_kinds == [contract.member_kind]
            });
        let required_shape = match contract.source {
            AggregateIdentitySource::Components { .. } => true,
            AggregateIdentitySource::Orbit(_) => source
                .relation_integrity
                .cardinality_contracts
                .iter()
                .any(|count| {
                    count.source_min == Some(1)
                        && count.source_max == Some(1)
                        && count.target_min == Some(1)
                        && count.target_max == Some(1)
                        && count.minimum_enforcement
                            == MinimumCardinalityEnforcement::CommitBoundary
                }),
        };
        homogeneous && required_shape && source.relation_integrity.aggregate_identity.is_none()
    })
}
