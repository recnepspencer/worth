//! Schema binding for Native-owned aggregate records requested by an installed contract.
pub(crate) mod private_sources;
mod source_binding;

mod storage;

use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::schema::data::{
    AggregateIdentityContractDeclaration, CardinalityContractDeclaration, ContractId,
    EndpointKindContractDeclaration, MinimumCardinalityEnforcement, PairMinimumSemantics,
    RelationIntegrityDeclarations, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
    SchemaRegistryError, SchemaVersionId,
};
pub(crate) use source_binding::member_domain;
pub use storage::register_aggregate_identity_storage;

/// Bind an installed graph definition to its Native-owned membership kind.
pub fn register_aggregate_membership(
    registry: RelationalSchemaRegistry,
    schema: &SchemaId,
    version: SchemaVersionId,
    contract: AggregateIdentityContractDeclaration,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let integrity = membership_integrity(&registry, &contract);
    let declarations = storage::membership_declarations(&registry, &contract);
    registry.register_relation_kind(RelationKindRegistration {
        kind_id: contract.membership_kind,
        kind_name: format!("worth-aggregate-membership-{}", contract.membership_kind.0),
        schema_id: schema.clone(),
        schema_version_id: version,
        cross_context_policy: CrossContextPolicy::AllowExplicit,
        cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
        aspect_contract_declarations: declarations,
        relation_integrity: integrity,
    })
}

fn membership_integrity(
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
) -> RelationIntegrityDeclarations {
    let mut integrity = endpoints(
        "aggregate-membership",
        contract.member_kind,
        contract.storage.aggregate_kind,
    );
    // One carrier may route distinct cases to different logical cohorts. Its
    // complete logical-key uniqueness is checked by the membership decoder.
    if member_domain(registry, contract).is_some_and(|domain| domain.entity_kind().is_some()) {
        integrity
            .cardinality_contracts
            .push(CardinalityContractDeclaration {
                contract_id: ContractId::new("aggregate-membership:one-owner"),
                source_max: Some(1),
                target_max: None,
                pair_max: None,
                source_min: None,
                target_min: None,
                pair_min: None,
                pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
                minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
            });
    }
    integrity.with_aggregate_identity(contract.clone())
}

pub(crate) fn validate_binding(
    registry: &RelationalSchemaRegistry,
    relation: &RelationKindRegistration,
) -> Result<(), SchemaRegistryError> {
    let Some(contract) = relation.relation_integrity.aggregate_identity.as_ref() else {
        return Ok(());
    };
    let invalid = || {
        SchemaRegistryError::invalid_relation_integrity_declaration(
        relation.kind_id, "aggregate identity must bind homogeneous graph sources and its exact reserved record grammar",
    )
    };
    let aggregate = registry
        .entity_kinds
        .get(&contract.storage.aggregate_kind)
        .ok_or_else(invalid)?;
    let lineage = registry
        .relation_kinds
        .get(&contract.storage.lineage_kind)
        .ok_or_else(invalid)?;
    let membership = relation;
    let mut expected_aggregate = aggregate.clone();
    expected_aggregate.aspect_contract_declarations =
        storage::aggregate_declarations(contract.storage);
    let expected_aggregate = crate::schema::canonicalize_entity_registration(expected_aggregate)?;
    let mut expected_lineage = lineage.clone();
    expected_lineage.aspect_contract_declarations = storage::lineage_declarations(contract.storage);
    expected_lineage.relation_integrity = endpoints(
        "aggregate-lineage",
        contract.storage.aggregate_kind,
        contract.storage.aggregate_kind,
    );
    expected_lineage.cross_context_policy = CrossContextPolicy::AllowExplicit;
    expected_lineage.cascade_delete_policy = CascadeDeletePolicy::RetainDanglingForAudit;
    let expected_lineage = crate::schema::canonicalize_relation_registration(expected_lineage)?;
    let mut expected_membership = membership.clone();
    expected_membership.aspect_contract_declarations =
        storage::membership_declarations(registry, contract);
    expected_membership.relation_integrity = membership_integrity(registry, contract);
    expected_membership.cross_context_policy = CrossContextPolicy::AllowExplicit;
    expected_membership.cascade_delete_policy = CascadeDeletePolicy::RetainDanglingForAudit;
    let expected_membership =
        crate::schema::canonicalize_relation_registration(expected_membership)?;
    let private = private_sources::with_candidate(registry, contract);
    if relation.kind_id != contract.membership_kind
        || !source_binding::valid(registry, contract, &private)
        || aggregate.aspect_contract_declarations != expected_aggregate.aspect_contract_declarations
        || lineage != &expected_lineage
        || membership != &expected_membership
        || lineage.relation_integrity.aggregate_identity.is_some()
        || registry
            .relation_kinds
            .values()
            .filter_map(|entry| entry.relation_integrity.aggregate_identity.as_ref())
            .any(|old| {
                old.definition == contract.definition
                    || old.membership_kind == contract.membership_kind
            })
        || !has_endpoints(
            membership,
            contract.member_kind,
            contract.storage.aggregate_kind,
        )
        || !has_endpoints(
            lineage,
            contract.storage.aggregate_kind,
            contract.storage.aggregate_kind,
        )
    {
        return Err(invalid());
    }
    // A later private-kind designation must not silently change an already
    // installed aggregate's ordinary source into consequence-dependent input.
    for old in registry
        .relation_kinds
        .values()
        .filter_map(|kind| kind.relation_integrity.aggregate_identity.as_ref())
    {
        if !source_binding::valid(registry, old, &private) {
            return Err(SchemaRegistryError::invalid_relation_integrity_declaration(
                relation.kind_id,
                format!(
                    "Native-private storage would invalidate aggregate source {:?}",
                    old.definition
                ),
            ));
        }
    }
    Ok(())
}

fn has_endpoints(
    relation: &RelationKindRegistration,
    from: crate::identity::data::KindId,
    to: crate::identity::data::KindId,
) -> bool {
    relation
        .relation_integrity
        .endpoint_kind_contracts
        .iter()
        .any(|endpoints| {
            endpoints.allowed_source_kinds == [from] && endpoints.allowed_target_kinds == [to]
        })
}

fn endpoints(
    name: &str,
    from: crate::identity::data::KindId,
    to: crate::identity::data::KindId,
) -> RelationIntegrityDeclarations {
    RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: ContractId::new(format!("{name}:endpoints")),
            allowed_source_kinds: vec![from],
            allowed_target_kinds: vec![to],
            self_edges_allowed: true,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
        }],
        vec![],
        vec![],
        vec![],
        vec![],
    )
}
