use super::lower_cross_context;
use std::collections::BTreeMap;
use worth_query_installation::facade::{
    ApplicationRelationDeletionPolicy, ApplicationRelationIntegrityRecord,
    ApplicationRelationUniquenessScope,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{
    AcyclicityContractDeclaration, AllowedCycleClass, CardinalityContractDeclaration,
    ConnectivityMinimumContractDeclaration, ConnectivityMinimumEnforcement, ContractId,
    DirectedTraversalKind, EndpointDeletionIntegrityDeclaration, EndpointDeletionIntegrityMode,
    EndpointKindContractDeclaration, MinimumCardinalityEnforcement, PairMinimumSemantics,
    PartitionIsolationContractDeclaration, PartitionIsolationMode, RelationIntegrityDeclarations,
    SymmetryContractDeclaration, SymmetryMode, UniquenessContractDeclaration, UniquenessScope,
};

pub(super) fn lower_relation_integrity(
    relation: &str,
    from_kind: KindId,
    to_kinds: Vec<KindId>,
    integrity: &ApplicationRelationIntegrityRecord,
    entity_kinds: Option<&BTreeMap<String, KindId>>,
) -> Result<RelationIntegrityDeclarations, super::WorthQueryPrimaryGraphInstallationDenial> {
    let endpoints = vec![EndpointKindContractDeclaration {
        contract_id: ContractId::new(format!("{relation}:endpoints")),
        allowed_source_kinds: vec![from_kind],
        allowed_target_kinds: to_kinds,
        self_edges_allowed: integrity.endpoints.self_edges_allowed,
        cross_context_policy: lower_cross_context(integrity.endpoints.cross_context_policy),
    }];
    let cardinality = integrity
        .cardinality
        .has_bound()
        .then(|| CardinalityContractDeclaration {
            contract_id: ContractId::new(format!("{relation}:cardinality")),
            source_max: integrity.cardinality.source_max,
            target_max: integrity.cardinality.target_max,
            pair_max: integrity.cardinality.pair_max,
            source_min: integrity.cardinality.source_min,
            target_min: integrity.cardinality.target_min,
            pair_min: integrity.cardinality.pair_min,
            pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
            minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
        });
    let deletion = lower_endpoint_deletion(relation, integrity.deletion);
    let acyclicity = integrity.acyclic.then(|| AcyclicityContractDeclaration {
        contract_id: ContractId::new(format!("{relation}:acyclic")),
        traversal_direction: DirectedTraversalKind::SourceToTarget,
        allowed_cycle_class: AllowedCycleClass::NoCycles,
    });
    let minimum = lower_minimum(relation, from_kind, integrity, entity_kinds)?;
    Ok(RelationIntegrityDeclarations::new(
        endpoints,
        cardinality.into_iter().collect(),
        integrity
            .unique
            .map(|scope| UniquenessContractDeclaration {
                contract_id: ContractId::new(format!("{relation}:unique")),
                scope: match scope {
                    ApplicationRelationUniquenessScope::DirectedSemanticEdge => {
                        UniquenessScope::DirectedSemanticEdge
                    }
                    ApplicationRelationUniquenessScope::NormalizedSymmetricEdge => {
                        UniquenessScope::NormalizedSymmetricEdge
                    }
                },
            })
            .into_iter()
            .collect(),
        integrity
            .symmetric
            .then(|| SymmetryContractDeclaration {
                contract_id: ContractId::new(format!("{relation}:symmetry")),
                mode: SymmetryMode::PairedTwinRequired,
            })
            .into_iter()
            .collect(),
        deletion.into_iter().collect(),
    )
    .with_acyclicity_contracts(acyclicity.into_iter().collect())
    .with_partition_isolation_contracts(
        integrity
            .same_partition_endpoints
            .then(|| PartitionIsolationContractDeclaration {
                contract_id: ContractId::new(format!("{relation}:same_partition_endpoints")),
                isolation_mode: PartitionIsolationMode::SamePartitionEndpoints,
            })
            .into_iter()
            .collect(),
    )
    .with_connectivity_minimum_contracts(minimum.into_iter().collect()))
}

fn lower_minimum(
    relation: &str,
    from_kind: KindId,
    integrity: &ApplicationRelationIntegrityRecord,
    entity_kinds: Option<&BTreeMap<String, KindId>>,
) -> Result<
    Option<ConnectivityMinimumContractDeclaration>,
    super::WorthQueryPrimaryGraphInstallationDenial,
> {
    use super::super::installation_primitives::{invalid_member, required_kind};
    let Some(minimum) = integrity.reaches_at_least.as_ref() else {
        return Ok(None);
    };
    let binding = entity_kinds.ok_or_else(|| invalid_member(relation))?;
    let mut target_kind_ids = minimum
        .target_kinds
        .iter()
        .map(|name| required_kind(binding, name))
        .collect::<Result<Vec<_>, _>>()?;
    target_kind_ids.sort_unstable();
    target_kind_ids.dedup();
    Ok(Some(ConnectivityMinimumContractDeclaration {
        contract_id: ContractId::new(format!("{relation}:reaches_at_least")),
        source_kind_ids: vec![from_kind],
        target_kind_ids,
        minimum_reachable_targets: minimum.minimum_reachable_targets,
        enforcement_boundary: ConnectivityMinimumEnforcement::SnapshotPublication,
    }))
}

fn lower_endpoint_deletion(
    relation: &str,
    value: ApplicationRelationDeletionPolicy,
) -> Option<EndpointDeletionIntegrityDeclaration> {
    let mode = match value {
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations => {
            EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations
        }
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit => {
            EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit
        }
        ApplicationRelationDeletionPolicy::RequireRelationRetirement => {
            EndpointDeletionIntegrityMode::RequireRelationRetirement
        }
        ApplicationRelationDeletionPolicy::RetainDanglingForAudit
        | ApplicationRelationDeletionPolicy::CascadeDeleteRelations => return None,
    };
    Some(EndpointDeletionIntegrityDeclaration {
        contract_id: ContractId::new(format!("{relation}:deletion")),
        mode,
    })
}

#[cfg(test)]
#[path = "relation_integrity/tests.rs"]
mod tests;
