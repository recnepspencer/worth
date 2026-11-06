//! One bounded lowering owner for configuration and request-funded targets.
use crate::config::data::CascadeDeletePolicy;
use crate::execution::RelationalExecutionDenialCause as Denial;
use crate::identity::data::KindId;
use crate::schema::{data::*, SchemaCompilationControl};

pub(super) fn lower_relation_integrity_plan(
    kind_id: KindId,
    declarations: &RelationIntegrityDeclarations,
    cascade_delete_policy: CascadeDeletePolicy,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<LoweredRelationIntegrityPlan, Denial> {
    let plan_revision = derive_relation_integrity_plan_revision_for_compilation(
        control,
        &declarations.endpoint_kind_contracts,
        &declarations.cardinality_contracts,
        &declarations.uniqueness_contracts,
        &declarations.symmetry_contracts,
        &declarations.endpoint_deletion_integrity_contracts,
        &declarations.acyclicity_contracts,
        &declarations.partition_isolation_contracts,
        &declarations.connectivity_minimum_contracts,
        declarations.aggregate_identity.as_ref(),
    )?;
    let mut source_count = 0usize;
    let mut target_count = 0usize;
    for endpoint in &declarations.endpoint_kind_contracts {
        control.checkpoint()?;
        source_count = source_count
            .checked_add(endpoint.allowed_source_kinds.len())
            .ok_or(Denial::MemoryOverflow)?;
        target_count = target_count
            .checked_add(endpoint.allowed_target_kinds.len())
            .ok_or(Denial::MemoryOverflow)?;
    }
    let mut candidate_source_kinds = Vec::with_capacity(source_count);
    let mut candidate_target_kinds = Vec::with_capacity(target_count);
    for endpoint in &declarations.endpoint_kind_contracts {
        control.checkpoint()?;
        candidate_source_kinds.extend_from_slice(&endpoint.allowed_source_kinds);
        candidate_target_kinds.extend_from_slice(&endpoint.allowed_target_kinds);
    }
    control.checkpoint()?;
    candidate_source_kinds.sort_unstable();
    candidate_source_kinds.dedup();
    control.checkpoint()?;
    candidate_target_kinds.sort_unstable();
    candidate_target_kinds.dedup();
    control.checkpoint()?;
    let mut plan = LoweredRelationIntegrityPlan {
        aggregate_identity: declarations.aggregate_identity.clone(),
        kind_id,
        plan_revision,
        endpoint_kind_contracts: Vec::with_capacity(declarations.endpoint_kind_contracts.len()),
        cardinality_maximum_contracts: Vec::with_capacity(declarations.cardinality_contracts.len()),
        cardinality_minimum_contracts: Vec::with_capacity(declarations.cardinality_contracts.len()),
        uniqueness_contracts: Vec::with_capacity(declarations.uniqueness_contracts.len()),
        symmetry_contracts: Vec::with_capacity(declarations.symmetry_contracts.len()),
        endpoint_deletion_integrity_contracts: Vec::with_capacity(
            declarations.endpoint_deletion_integrity_contracts.len(),
        ),
        acyclicity_contracts: Vec::with_capacity(declarations.acyclicity_contracts.len()),
        partition_isolation_contracts: Vec::with_capacity(
            declarations.partition_isolation_contracts.len(),
        ),
        connectivity_minimum_contracts: Vec::with_capacity(
            declarations.connectivity_minimum_contracts.len(),
        ),
    };
    for declaration in &declarations.endpoint_kind_contracts {
        control.checkpoint()?;
        plan.endpoint_kind_contracts
            .push(LoweredEndpointKindContract {
                contract_id: declaration.contract_id.clone(),
                relation_kind_id: kind_id,
                allowed_source_kinds: declaration.allowed_source_kinds.clone(),
                allowed_target_kinds: declaration.allowed_target_kinds.clone(),
                self_edges_allowed: declaration.self_edges_allowed,
                cross_context_policy: declaration.cross_context_policy,
                plan_revision,
            });
    }
    for declaration in &declarations.cardinality_contracts {
        control.checkpoint()?;
        if declaration.source_max.is_some()
            || declaration.target_max.is_some()
            || declaration.pair_max.is_some()
        {
            plan.cardinality_maximum_contracts
                .push(LoweredCardinalityMaximumContract {
                    contract_id: declaration.contract_id.clone(),
                    relation_kind_id: kind_id,
                    source_max: declaration.source_max,
                    target_max: declaration.target_max,
                    pair_max: declaration.pair_max,
                    plan_revision,
                });
        }
        if declaration.source_min.is_some()
            || declaration.target_min.is_some()
            || declaration.pair_min.is_some()
        {
            plan.cardinality_minimum_contracts
                .push(LoweredCardinalityMinimumContract {
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
                });
        }
    }
    for declaration in &declarations.uniqueness_contracts {
        control.checkpoint()?;
        plan.uniqueness_contracts.push(LoweredUniquenessContract {
            contract_id: declaration.contract_id.clone(),
            relation_kind_id: kind_id,
            scope: declaration.scope,
            plan_revision,
        });
    }
    for declaration in &declarations.symmetry_contracts {
        control.checkpoint()?;
        plan.symmetry_contracts.push(LoweredSymmetryContract {
            contract_id: declaration.contract_id.clone(),
            relation_kind_id: kind_id,
            mode: declaration.mode,
            plan_revision,
        });
    }
    for declaration in &declarations.endpoint_deletion_integrity_contracts {
        control.checkpoint()?;
        plan.endpoint_deletion_integrity_contracts
            .push(LoweredEndpointDeletionIntegrityContract {
                contract_id: declaration.contract_id.clone(),
                relation_kind_id: kind_id,
                mode: declaration.mode,
                cascade_delete_policy,
                plan_revision,
            });
    }
    for declaration in &declarations.acyclicity_contracts {
        control.checkpoint()?;
        plan.acyclicity_contracts.push(LoweredAcyclicityContract {
            contract_id: declaration.contract_id.clone(),
            relation_kind_id: kind_id,
            traversal_direction: declaration.traversal_direction,
            allowed_cycle_class: declaration.allowed_cycle_class,
            plan_revision,
        });
    }
    for declaration in &declarations.partition_isolation_contracts {
        control.checkpoint()?;
        plan.partition_isolation_contracts
            .push(LoweredPartitionIsolationContract {
                contract_id: declaration.contract_id.clone(),
                relation_kind_id: kind_id,
                isolation_mode: declaration.isolation_mode,
                plan_revision,
            });
    }
    for declaration in &declarations.connectivity_minimum_contracts {
        control.checkpoint()?;
        plan.connectivity_minimum_contracts
            .push(LoweredConnectivityMinimumContract {
                contract_id: declaration.contract_id.clone(),
                source_kind_ids: declaration.source_kind_ids.clone(),
                relation_kind_id: kind_id,
                target_kind_ids: declaration.target_kind_ids.clone(),
                minimum_reachable_targets: declaration.minimum_reachable_targets,
                enforcement_boundary: declaration.enforcement_boundary,
                plan_revision,
            });
    }
    control.checkpoint()?;
    Ok(plan)
}
