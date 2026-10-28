//! Installed graph sources share member meaning, never private storage roles.
use crate::schema::data::{
    AggregateIdentityContractDeclaration, AggregateIdentitySource, MinimumCardinalityEnforcement,
    RelationalSchemaRegistry,
};
use std::collections::BTreeSet;

pub(super) fn valid(
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
    private: &BTreeSet<crate::identity::data::KindId>,
) -> bool {
    let reserved = [
        contract.member_kind,
        contract.membership_kind,
        contract.storage.aggregate_kind,
        contract.storage.lineage_kind,
    ];
    if private.contains(&contract.member_kind)
        || reserved.into_iter().collect::<BTreeSet<_>>().len() != reserved.len()
    {
        return false;
    }
    if let Some(identity) = contract.source.derived_relation() {
        let Some(source) = registry.derived_relation(identity) else {
            return false;
        };
        let parts = source.parts();
        let domain = &parts.from;
        let Some(carrier) = carrier_kind(registry, domain) else {
            return false;
        };
        let projected = contract.storage.projected;
        let valid_storage = domain.entity_kind().is_some()
            || projected.is_some_and(|storage| {
                let identities = [
                    storage.coordinate_aspect,
                    storage.membership_aspect,
                    contract.storage.aggregate_aspect,
                    contract.storage.lineage_aspect,
                ];
                identities.into_iter().collect::<BTreeSet<_>>().len() == identities.len()
            });
        return parts.from == parts.to
            && carrier == contract.member_kind
            && valid_storage
            && source
                .physical_sources()
                .iter()
                .all(|(relation, from, to)| {
                    !private.contains(relation) && !private.contains(from) && !private.contains(to)
                })
            && source
                .entry
                .carriers
                .iter()
                .all(|carrier| !private.contains(carrier))
            && match &contract.source {
                AggregateIdentitySource::DerivedComponents { .. } => true,
                AggregateIdentitySource::DerivedOrbit { .. } => {
                    parts.laws.as_ref().is_some_and(|laws| laws.orbit)
                }
                _ => false,
            };
    }
    let Some(relations) = contract.source.stored_relations() else {
        return false;
    };
    if relations.is_empty() {
        return false;
    }
    relations.iter().all(|kind| {
        if private.contains(kind) {
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
            AggregateIdentitySource::DerivedOrbit { .. }
            | AggregateIdentitySource::DerivedComponents { .. } => false,
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

/// The actual registered endpoint domain; the contract's KindId is only its
/// physical carrier and must never be treated as a logical member coordinate.
pub(crate) fn member_domain(
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
) -> Option<crate::graph_member::NativeMemberDomain> {
    match contract.source.derived_relation() {
        Some(identity) => registry
            .derived_relation(identity)
            .map(|source| source.parts().from.clone()),
        None => Some(contract.member_kind.into()),
    }
}

fn carrier_kind(
    registry: &RelationalSchemaRegistry,
    domain: &crate::graph_member::NativeMemberDomain,
) -> Option<crate::identity::data::KindId> {
    match domain {
        crate::graph_member::NativeMemberDomain::Entity(kind) => Some(*kind),
        crate::graph_member::NativeMemberDomain::Projected { projected } => registry
            .projected_member_domain(projected)
            .map(|domain| domain.parts().carrier),
    }
}
