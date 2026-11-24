use contracts::durability::data::recovery_errors::{
    RecoveryAuthorityContinuityMismatch, RelationIntegrityContractFamily,
};
use contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn schema_registry_mismatch(
    expected: &contracts::schema::data::RelationalSchemaRegistry,
    found: &contracts::schema::data::RelationalSchemaRegistry,
    expected_primary_schema_version: contracts::schema::data::SchemaVersionId,
    found_primary_schema_version: contracts::schema::data::SchemaVersionId,
) -> RecoveryAuthorityContinuityMismatch {
    for (kind_id, expected_registration) in expected.owner_entity_kinds() {
        let Some(found_registration) = (*found.owner_entity_kinds()).get(kind_id) else {
            break;
        };
        if expected_registration.kind_name == found_registration.kind_name
            && expected_registration.schema_id == found_registration.schema_id
            && expected_registration.schema_version_id == found_registration.schema_version_id
            && expected_registration
                .aspect_contract_declarations
                .plan_revision
                != found_registration
                    .aspect_contract_declarations
                    .plan_revision
        {
            return RecoveryAuthorityContinuityMismatch::EntityAspectPlanRevision {
                kind_id: *kind_id,
                kind_name: expected_registration.kind_name.clone(),
                expected_revision: expected_registration
                    .aspect_contract_declarations
                    .plan_revision
                    .0,
                found_revision: found_registration
                    .aspect_contract_declarations
                    .plan_revision
                    .0,
            };
        }
    }
    for (kind_id, expected_registration) in expected.owner_relation_kinds() {
        let Some(found_registration) = (*found.owner_relation_kinds()).get(kind_id) else {
            break;
        };
        if expected_registration.kind_name == found_registration.kind_name
            && expected_registration.schema_id == found_registration.schema_id
            && expected_registration.schema_version_id == found_registration.schema_version_id
            && expected_registration
                .aspect_contract_declarations
                .plan_revision
                != found_registration
                    .aspect_contract_declarations
                    .plan_revision
        {
            return RecoveryAuthorityContinuityMismatch::RelationAspectPlanRevision {
                kind_id: *kind_id,
                kind_name: expected_registration.kind_name.clone(),
                expected_revision: expected_registration
                    .aspect_contract_declarations
                    .plan_revision
                    .0,
                found_revision: found_registration
                    .aspect_contract_declarations
                    .plan_revision
                    .0,
            };
        }
        if expected_registration.kind_name == found_registration.kind_name
            && expected_registration.schema_id == found_registration.schema_id
            && expected_registration.schema_version_id == found_registration.schema_version_id
            && expected_registration.relation_integrity.plan_revision
                != found_registration.relation_integrity.plan_revision
        {
            let (contract_family, expected_contract_ids, found_contract_ids) =
                relation_integrity_contract_mismatch(
                    &expected_registration.relation_integrity,
                    &found_registration.relation_integrity,
                );
            return RecoveryAuthorityContinuityMismatch::RelationIntegrityPlanRevision {
                kind_id: *kind_id,
                kind_name: expected_registration.kind_name.clone(),
                contract_family,
                expected_revision: expected_registration.relation_integrity.plan_revision.0,
                found_revision: found_registration.relation_integrity.plan_revision.0,
                expected_contract_ids,
                found_contract_ids,
            };
        }
    }
    RecoveryAuthorityContinuityMismatch::SchemaRegistryShape {
        expected_primary_schema_version,
        found_primary_schema_version,
        expected_entity_kind_count: (*expected.owner_entity_kinds()).len(),
        found_entity_kind_count: (*found.owner_entity_kinds()).len(),
        expected_relation_kind_count: (*expected.owner_relation_kinds()).len(),
        found_relation_kind_count: (*found.owner_relation_kinds()).len(),
    }
}

fn relation_integrity_contract_mismatch(
    expected: &contracts::schema::data::relation_integrity::declarations::RelationIntegrityDeclarations,
    found: &contracts::schema::data::relation_integrity::declarations::RelationIntegrityDeclarations,
) -> (
    RelationIntegrityContractFamily,
    Vec<contracts::schema::data::relation_integrity::contract_id::ContractId>,
    Vec<contracts::schema::data::relation_integrity::contract_id::ContractId>,
) {
    if expected.endpoint_kind_contracts != found.endpoint_kind_contracts {
        return (
            RelationIntegrityContractFamily::EndpointKind,
            expected
                .endpoint_kind_contracts
                .iter()
                .map(|contract| contract.contract_id.clone())
                .collect(),
            found
                .endpoint_kind_contracts
                .iter()
                .map(|contract| contract.contract_id.clone())
                .collect(),
        );
    }
    if expected.cardinality_contracts != found.cardinality_contracts {
        return (
            RelationIntegrityContractFamily::Cardinality,
            expected
                .cardinality_contracts
                .iter()
                .map(|contract| contract.contract_id.clone())
                .collect(),
            found
                .cardinality_contracts
                .iter()
                .map(|contract| contract.contract_id.clone())
                .collect(),
        );
    }
    if expected.uniqueness_contracts != found.uniqueness_contracts {
        return (
            RelationIntegrityContractFamily::Uniqueness,
            expected
                .uniqueness_contracts
                .iter()
                .map(|contract| contract.contract_id.clone())
                .collect(),
            found
                .uniqueness_contracts
                .iter()
                .map(|contract| contract.contract_id.clone())
                .collect(),
        );
    }
    if expected.symmetry_contracts != found.symmetry_contracts {
        return (
            RelationIntegrityContractFamily::Symmetry,
            expected
                .symmetry_contracts
                .iter()
                .map(|contract| contract.contract_id.clone())
                .collect(),
            found
                .symmetry_contracts
                .iter()
                .map(|contract| contract.contract_id.clone())
                .collect(),
        );
    }
    if expected.endpoint_deletion_integrity_contracts != found.endpoint_deletion_integrity_contracts
    {
        return (
            RelationIntegrityContractFamily::EndpointDeletionIntegrity,
            expected
                .endpoint_deletion_integrity_contracts
                .iter()
                .map(|contract| contract.contract_id.clone())
                .collect(),
            found
                .endpoint_deletion_integrity_contracts
                .iter()
                .map(|contract| contract.contract_id.clone())
                .collect(),
        );
    }
    let mut expected_contract_ids = expected
        .acyclicity_contracts
        .iter()
        .map(|contract| contract.contract_id.clone())
        .collect::<Vec<_>>();
    expected_contract_ids.extend(
        expected
            .partition_isolation_contracts
            .iter()
            .map(|contract| contract.contract_id.clone()),
    );
    expected_contract_ids.extend(
        expected
            .connectivity_minimum_contracts
            .iter()
            .map(|contract| contract.contract_id.clone()),
    );

    let mut found_contract_ids = found
        .acyclicity_contracts
        .iter()
        .map(|contract| contract.contract_id.clone())
        .collect::<Vec<_>>();
    found_contract_ids.extend(
        found
            .partition_isolation_contracts
            .iter()
            .map(|contract| contract.contract_id.clone()),
    );
    found_contract_ids.extend(
        found
            .connectivity_minimum_contracts
            .iter()
            .map(|contract| contract.contract_id.clone()),
    );

    (
        RelationIntegrityContractFamily::Aggregate,
        expected_contract_ids,
        found_contract_ids,
    )
}
