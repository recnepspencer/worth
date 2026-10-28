use std::collections::BTreeSet;

use crate::schema::data::{
    derive_relation_integrity_plan_revision, LoweredCardinalityMaximumContract,
    LoweredCardinalityMinimumContract, LoweredEndpointDeletionIntegrityContract,
    LoweredEndpointKindContract, LoweredPartitionIsolationContract, LoweredRelationIntegrityPlan,
    LoweredSymmetryContract, LoweredUniquenessContract, RelationIntegrityDeclarations,
};

pub(super) fn lower_relation_integrity_plan(
    kind_id: crate::identity::data::KindId,
    declarations: &RelationIntegrityDeclarations,
    cascade_delete_policy: crate::config::data::CascadeDeletePolicy,
) -> LoweredRelationIntegrityPlan {
    let plan_revision = derive_relation_integrity_plan_revision(
        &declarations.endpoint_kind_contracts,
        &declarations.cardinality_contracts,
        &declarations.uniqueness_contracts,
        &declarations.symmetry_contracts,
        &declarations.endpoint_deletion_integrity_contracts,
        &declarations.acyclicity_contracts,
        &declarations.partition_isolation_contracts,
        &declarations.connectivity_minimum_contracts,
        declarations.aggregate_identity.as_ref(),
    );
    let candidate_source_kinds = declarations
        .endpoint_kind_contracts
        .iter()
        .flat_map(|declaration| declaration.allowed_source_kinds.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let candidate_target_kinds = declarations
        .endpoint_kind_contracts
        .iter()
        .flat_map(|declaration| declaration.allowed_target_kinds.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    LoweredRelationIntegrityPlan {
        aggregate_identity: declarations.aggregate_identity.clone(),
        kind_id,
        plan_revision,
        endpoint_kind_contracts: declarations
            .endpoint_kind_contracts
            .iter()
            .map(|declaration| LoweredEndpointKindContract {
                contract_id: declaration.contract_id.clone(),
                relation_kind_id: kind_id,
                allowed_source_kinds: declaration.allowed_source_kinds.clone(),
                allowed_target_kinds: declaration.allowed_target_kinds.clone(),
                self_edges_allowed: declaration.self_edges_allowed,
                cross_context_policy: declaration.cross_context_policy,
                plan_revision,
            })
            .collect(),
        cardinality_maximum_contracts: declarations
            .cardinality_contracts
            .iter()
            .filter(|declaration| {
                declaration.source_max.is_some()
                    || declaration.target_max.is_some()
                    || declaration.pair_max.is_some()
            })
            .map(|declaration| LoweredCardinalityMaximumContract {
                contract_id: declaration.contract_id.clone(),
                relation_kind_id: kind_id,
                source_max: declaration.source_max,
                target_max: declaration.target_max,
                pair_max: declaration.pair_max,
                plan_revision,
            })
            .collect(),
        cardinality_minimum_contracts: declarations
            .cardinality_contracts
            .iter()
            .filter(|declaration| {
                declaration.source_min.is_some()
                    || declaration.target_min.is_some()
                    || declaration.pair_min.is_some()
            })
            .map(|declaration| LoweredCardinalityMinimumContract {
                contract_id: declaration.contract_id.clone(),
                relation_kind_id: kind_id,
                source_min: declaration.source_min,
                target_min: declaration.target_min,
                pair_min: declaration.pair_min,
                pair_min_semantics: declaration.pair_min_semantics,
                candidate_source_kinds: candidate_source_kinds.clone(),
                candidate_target_kinds: candidate_target_kinds.clone(),
                minimum_enforcement: declaration.minimum_enforcement,
                plan_revision,
            })
            .collect(),
        uniqueness_contracts: declarations
            .uniqueness_contracts
            .iter()
            .map(|declaration| LoweredUniquenessContract {
                contract_id: declaration.contract_id.clone(),
                relation_kind_id: kind_id,
                scope: declaration.scope,
                plan_revision,
            })
            .collect(),
        symmetry_contracts: declarations
            .symmetry_contracts
            .iter()
            .map(|declaration| LoweredSymmetryContract {
                contract_id: declaration.contract_id.clone(),
                relation_kind_id: kind_id,
                mode: declaration.mode,
                plan_revision,
            })
            .collect(),
        endpoint_deletion_integrity_contracts: declarations
            .endpoint_deletion_integrity_contracts
            .iter()
            .map(|declaration| LoweredEndpointDeletionIntegrityContract {
                contract_id: declaration.contract_id.clone(),
                relation_kind_id: kind_id,
                mode: declaration.mode,
                cascade_delete_policy,
                plan_revision,
            })
            .collect(),
        acyclicity_contracts: declarations
            .acyclicity_contracts
            .iter()
            .map(
                |declaration| crate::schema::data::LoweredAcyclicityContract {
                    contract_id: declaration.contract_id.clone(),
                    relation_kind_id: kind_id,
                    traversal_direction: declaration.traversal_direction,
                    allowed_cycle_class: declaration.allowed_cycle_class,
                    plan_revision,
                },
            )
            .collect(),
        partition_isolation_contracts: declarations
            .partition_isolation_contracts
            .iter()
            .map(|declaration| LoweredPartitionIsolationContract {
                contract_id: declaration.contract_id.clone(),
                relation_kind_id: kind_id,
                isolation_mode: declaration.isolation_mode,
                plan_revision,
            })
            .collect(),
        connectivity_minimum_contracts: declarations
            .connectivity_minimum_contracts
            .iter()
            .map(
                |declaration| crate::schema::data::LoweredConnectivityMinimumContract {
                    contract_id: declaration.contract_id.clone(),
                    source_kind_ids: declaration.source_kind_ids.clone(),
                    relation_kind_id: kind_id,
                    target_kind_ids: declaration.target_kind_ids.clone(),
                    minimum_reachable_targets: declaration.minimum_reachable_targets,
                    enforcement_boundary: declaration.enforcement_boundary,
                    plan_revision,
                },
            )
            .collect(),
    }
}
