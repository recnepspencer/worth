use std::collections::BTreeSet;

use crate::schema::data::{
    derive_relation_integrity_plan_revision, AllowedCycleClass, CardinalityContractDeclaration,
    ConnectivityMinimumEnforcement, ContractId, DirectedTraversalKind,
    EndpointKindContractDeclaration, MinimumCardinalityEnforcement, PairMinimumSemantics,
    PartitionIsolationMode, RelationIntegrityDeclarations, SchemaRegistryError,
};

pub(super) fn canonicalize_relation_integrity_declarations(
    kind_id: crate::identity::data::KindId,
    declarations: RelationIntegrityDeclarations,
) -> Result<RelationIntegrityDeclarations, SchemaRegistryError> {
    let mut seen_contract_ids = BTreeSet::new();
    let mut endpoint_kind_contracts = declarations.endpoint_kind_contracts;
    let mut cardinality_contracts = declarations.cardinality_contracts;
    let mut uniqueness_contracts = declarations.uniqueness_contracts;
    let mut symmetry_contracts = declarations.symmetry_contracts;
    let mut endpoint_deletion_integrity_contracts =
        declarations.endpoint_deletion_integrity_contracts;
    let mut acyclicity_contracts = declarations.acyclicity_contracts;
    let mut partition_isolation_contracts = declarations.partition_isolation_contracts;
    let mut connectivity_minimum_contracts = declarations.connectivity_minimum_contracts;

    endpoint_kind_contracts.sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    cardinality_contracts.sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    uniqueness_contracts.sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    symmetry_contracts.sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    endpoint_deletion_integrity_contracts
        .sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    acyclicity_contracts.sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    partition_isolation_contracts.sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    connectivity_minimum_contracts.sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    for declaration in &mut endpoint_kind_contracts {
        declaration.allowed_source_kinds.sort();
        declaration.allowed_source_kinds.dedup();
        declaration.allowed_target_kinds.sort();
        declaration.allowed_target_kinds.dedup();
    }

    for declaration in &endpoint_kind_contracts {
        validate_contract_id(kind_id, &declaration.contract_id, &mut seen_contract_ids)?;
        validate_endpoint_kind_contract(kind_id, declaration)?;
    }
    for declaration in &cardinality_contracts {
        validate_contract_id(kind_id, &declaration.contract_id, &mut seen_contract_ids)?;
        validate_cardinality_contract(kind_id, declaration, &endpoint_kind_contracts)?;
    }
    for declaration in &uniqueness_contracts {
        validate_contract_id(kind_id, &declaration.contract_id, &mut seen_contract_ids)?;
    }
    for declaration in &symmetry_contracts {
        validate_contract_id(kind_id, &declaration.contract_id, &mut seen_contract_ids)?;
    }
    for declaration in &endpoint_deletion_integrity_contracts {
        validate_contract_id(kind_id, &declaration.contract_id, &mut seen_contract_ids)?;
    }
    for declaration in &acyclicity_contracts {
        validate_contract_id(kind_id, &declaration.contract_id, &mut seen_contract_ids)?;
        match declaration.traversal_direction {
            DirectedTraversalKind::SourceToTarget => {}
        }
        match declaration.allowed_cycle_class {
            AllowedCycleClass::NoCycles => {}
        }
    }
    for declaration in &partition_isolation_contracts {
        validate_contract_id(kind_id, &declaration.contract_id, &mut seen_contract_ids)?;
        match declaration.isolation_mode {
            PartitionIsolationMode::SamePartitionEndpoints => {}
        }
    }
    for declaration in &mut connectivity_minimum_contracts {
        validate_contract_id(kind_id, &declaration.contract_id, &mut seen_contract_ids)?;
        declaration.source_kind_ids.sort();
        declaration.source_kind_ids.dedup();
        declaration.target_kind_ids.sort();
        declaration.target_kind_ids.dedup();
        if declaration.source_kind_ids.is_empty() || declaration.target_kind_ids.is_empty() {
            return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
                kind_id,
                format!(
                    "connectivity minimum contract '{}' must declare source and target kind domains",
                    declaration.contract_id
                ),
            ));
        }
        if declaration.minimum_reachable_targets == 0 {
            return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
                kind_id,
                format!(
                    "connectivity minimum contract '{}' must require at least one reachable target",
                    declaration.contract_id
                ),
            ));
        }
        match declaration.enforcement_boundary {
            ConnectivityMinimumEnforcement::SnapshotPublication => {}
        }
    }

    Ok(RelationIntegrityDeclarations {
        plan_revision: derive_relation_integrity_plan_revision(
            &endpoint_kind_contracts,
            &cardinality_contracts,
            &uniqueness_contracts,
            &symmetry_contracts,
            &endpoint_deletion_integrity_contracts,
            &acyclicity_contracts,
            &partition_isolation_contracts,
            &connectivity_minimum_contracts,
        ),
        endpoint_kind_contracts,
        cardinality_contracts,
        uniqueness_contracts,
        symmetry_contracts,
        endpoint_deletion_integrity_contracts,
        acyclicity_contracts,
        partition_isolation_contracts,
        connectivity_minimum_contracts,
    })
}

fn validate_contract_id(
    kind_id: crate::identity::data::KindId,
    contract_id: &ContractId,
    seen_contract_ids: &mut BTreeSet<ContractId>,
) -> Result<(), SchemaRegistryError> {
    if contract_id.trim().is_empty() {
        return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
            kind_id,
            "relation contract id must not be empty",
        ));
    }
    if !seen_contract_ids.insert(contract_id.clone()) {
        return Err(SchemaRegistryError::duplicate_relation_contract_id(
            kind_id,
            contract_id.clone(),
        ));
    }
    Ok(())
}

fn validate_endpoint_kind_contract(
    kind_id: crate::identity::data::KindId,
    declaration: &EndpointKindContractDeclaration,
) -> Result<(), SchemaRegistryError> {
    if declaration.allowed_source_kinds.is_empty() {
        return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
            kind_id,
            format!(
                "endpoint kind contract '{}' must declare at least one allowed source kind",
                declaration.contract_id
            ),
        ));
    }
    if declaration.allowed_target_kinds.is_empty() {
        return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
            kind_id,
            format!(
                "endpoint kind contract '{}' must declare at least one allowed target kind",
                declaration.contract_id
            ),
        ));
    }
    Ok(())
}

fn validate_cardinality_contract(
    kind_id: crate::identity::data::KindId,
    declaration: &CardinalityContractDeclaration,
    endpoint_kind_contracts: &[EndpointKindContractDeclaration],
) -> Result<(), SchemaRegistryError> {
    if declaration.source_max.is_none()
        && declaration.target_max.is_none()
        && declaration.pair_max.is_none()
        && declaration.source_min.is_none()
        && declaration.target_min.is_none()
        && declaration.pair_min.is_none()
    {
        return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
            kind_id,
            format!(
                "cardinality contract '{}' must declare at least one bound",
                declaration.contract_id
            ),
        ));
    }
    if declaration.source_min == Some(0)
        || declaration.target_min == Some(0)
        || declaration.pair_min == Some(0)
    {
        return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
            kind_id,
            format!(
                "cardinality contract '{}' minimums must be greater than zero",
                declaration.contract_id
            ),
        ));
    }
    if declaration.pair_max == Some(0) {
        return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
            kind_id,
            format!(
                "cardinality contract '{}' must not declare pair_max = 0",
                declaration.contract_id
            ),
        ));
    }
    if declaration.pair_min.is_some() {
        match declaration.pair_min_semantics {
            PairMinimumSemantics::ObservedDirectedPairs => {}
        }
    }
    for (minimum, maximum, label) in [
        (declaration.source_min, declaration.source_max, "source"),
        (declaration.target_min, declaration.target_max, "target"),
        (declaration.pair_min, declaration.pair_max, "pair"),
    ] {
        if let (Some(minimum), Some(maximum)) = (minimum, maximum)
            && minimum > maximum
        {
            return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
                kind_id,
                format!(
                    "cardinality contract '{}' declares {label}_min > {label}_max",
                    declaration.contract_id
                ),
            ));
        }
    }
    match declaration.minimum_enforcement {
        MinimumCardinalityEnforcement::CommitBoundary
        | MinimumCardinalityEnforcement::CertificationBoundary => {}
    }
    if (declaration.source_min.is_some() || declaration.target_min.is_some())
        && endpoint_kind_contracts.is_empty()
    {
        return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
            kind_id,
            format!(
                "cardinality contract '{}' requires endpoint kind contracts to define minimum candidate domains",
                declaration.contract_id
            ),
        ));
    }
    Ok(())
}
