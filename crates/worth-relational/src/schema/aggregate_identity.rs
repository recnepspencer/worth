//! Schema binding for Native-owned aggregate records requested by an installed contract.
use crate::aggregate_identity::record::{
    AGGREGATE_ASPECT, AGGREGATE_FIELDS, LINEAGE_ASPECT, LINEAGE_FIELDS,
};
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::schema::data::{
    AggregateIdentityContractDeclaration, AggregateIdentityStorage, AspectBinding,
    CardinalityContractDeclaration, ContractId, DeclaredAspectContractBinding,
    EndpointKindContractDeclaration, EntityKindRegistration, KindAspectContractDeclarations,
    MinimumCardinalityEnforcement, PairMinimumSemantics, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaRegistryError,
    SchemaVersionId,
};
use worth_foundational::facade::{aspects, AspectIdentity, AspectKey, FieldKey, ScalarAspectType};

/// Register the exact private record grammar used by Native aggregate ownership.
pub fn register_aggregate_identity_storage(
    registry: RelationalSchemaRegistry,
    schema: &SchemaId,
    version: SchemaVersionId,
    storage: AggregateIdentityStorage,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let registry = registry.register_entity_kind(EntityKindRegistration {
        kind_id: storage.aggregate_kind,
        kind_name: "worth-aggregate-identity".into(),
        schema_id: schema.clone(),
        schema_version_id: version,
        aspect_contract_declarations: declarations(
            AGGREGATE_ASPECT,
            storage.aggregate_aspect,
            AGGREGATE_FIELDS,
            false,
        ),
    })?;
    registry.register_relation_kind(RelationKindRegistration {
        kind_id: storage.lineage_kind,
        kind_name: "worth-aggregate-lineage".into(),
        schema_id: schema.clone(),
        schema_version_id: version,
        cross_context_policy: CrossContextPolicy::AllowExplicit,
        cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
        aspect_contract_declarations: declarations(
            LINEAGE_ASPECT,
            storage.lineage_aspect,
            LINEAGE_FIELDS,
            true,
        ),
        relation_integrity: endpoints(
            "aggregate-lineage",
            storage.aggregate_kind,
            storage.aggregate_kind,
        ),
    })
}

/// Bind an installed permutation definition to its Native-owned membership kind.
pub fn register_aggregate_membership(
    registry: RelationalSchemaRegistry,
    schema: &SchemaId,
    version: SchemaVersionId,
    contract: AggregateIdentityContractDeclaration,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let integrity = membership_integrity(contract);
    registry.register_relation_kind(RelationKindRegistration {
        kind_id: contract.membership_kind,
        kind_name: format!("worth-aggregate-membership-{}", contract.membership_kind.0),
        schema_id: schema.clone(),
        schema_version_id: version,
        cross_context_policy: CrossContextPolicy::AllowExplicit,
        cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
        aspect_contract_declarations: Default::default(),
        relation_integrity: integrity,
    })
}

fn membership_integrity(
    contract: AggregateIdentityContractDeclaration,
) -> RelationIntegrityDeclarations {
    let mut integrity = endpoints(
        "aggregate-membership",
        contract.member_kind,
        contract.storage.aggregate_kind,
    );
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
    integrity.with_aggregate_identity(contract)
}

pub(crate) fn validate_binding(
    registry: &RelationalSchemaRegistry,
    relation: &RelationKindRegistration,
) -> Result<(), SchemaRegistryError> {
    let Some(contract) = relation.relation_integrity.aggregate_identity else {
        return Ok(());
    };
    let invalid = || {
        SchemaRegistryError::invalid_relation_integrity_declaration(
        relation.kind_id, "aggregate identity must bind a homogeneous permutation and its exact reserved record grammar",
    )
    };
    let source = registry
        .relation_kinds
        .get(&contract.source_relation)
        .ok_or_else(invalid)?;
    let homogeneous = source
        .relation_integrity
        .endpoint_kind_contracts
        .iter()
        .any(|endpoints| {
            endpoints.allowed_source_kinds == [contract.member_kind]
                && endpoints.allowed_target_kinds == [contract.member_kind]
        });
    let exact = source
        .relation_integrity
        .cardinality_contracts
        .iter()
        .any(|count| {
            count.source_min == Some(1)
                && count.source_max == Some(1)
                && count.target_min == Some(1)
                && count.target_max == Some(1)
                && count.minimum_enforcement == MinimumCardinalityEnforcement::CommitBoundary
        });
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
    expected_aggregate.aspect_contract_declarations = declarations(
        AGGREGATE_ASPECT,
        contract.storage.aggregate_aspect,
        AGGREGATE_FIELDS,
        false,
    );
    let expected_aggregate = crate::schema::canonicalize_entity_registration(expected_aggregate)?;
    let mut expected_lineage = lineage.clone();
    expected_lineage.aspect_contract_declarations = declarations(
        LINEAGE_ASPECT,
        contract.storage.lineage_aspect,
        LINEAGE_FIELDS,
        true,
    );
    expected_lineage.relation_integrity = endpoints(
        "aggregate-lineage",
        contract.storage.aggregate_kind,
        contract.storage.aggregate_kind,
    );
    expected_lineage.cross_context_policy = CrossContextPolicy::AllowExplicit;
    expected_lineage.cascade_delete_policy = CascadeDeletePolicy::RetainDanglingForAudit;
    let expected_lineage = crate::schema::canonicalize_relation_registration(expected_lineage)?;
    let mut expected_membership = membership.clone();
    expected_membership.aspect_contract_declarations = Default::default();
    expected_membership.relation_integrity = membership_integrity(contract);
    expected_membership.cross_context_policy = CrossContextPolicy::AllowExplicit;
    expected_membership.cascade_delete_policy = CascadeDeletePolicy::RetainDanglingForAudit;
    let expected_membership =
        crate::schema::canonicalize_relation_registration(expected_membership)?;
    let kinds = [
        contract.source_relation,
        contract.member_kind,
        contract.storage.aggregate_kind,
        contract.storage.lineage_kind,
        contract.membership_kind,
    ];
    let distinct: std::collections::BTreeSet<_> = kinds.into_iter().collect();
    if relation.kind_id != contract.membership_kind
        || !homogeneous
        || !exact
        || distinct.len() != kinds.len()
        || aggregate.aspect_contract_declarations != expected_aggregate.aspect_contract_declarations
        || lineage != &expected_lineage
        || membership != &expected_membership
        || lineage.relation_integrity.aggregate_identity.is_some()
        || source.relation_integrity.aggregate_identity.is_some()
        || !membership
            .relation_integrity
            .cardinality_contracts
            .iter()
            .any(|count| count.source_max == Some(1))
        || registry
            .relation_kinds
            .values()
            .filter_map(|entry| entry.relation_integrity.aggregate_identity)
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

fn declarations(
    aspect: &str,
    identity: AspectIdentity,
    fields: &[(&str, ScalarAspectType)],
    relation: bool,
) -> KindAspectContractDeclarations {
    let &(first, scalar) = fields.first().expect("fixed aggregate grammar is nonempty");
    let mut shape = aspects().struct_fields().required(first, scalar);
    for &(field, kind) in &fields[1..] {
        shape = shape.required(field, kind);
    }
    let contract = aspects()
        .contract()
        .for_key(AspectKey::new(aspect).expect("fixed aggregate aspect"))
        .identified_by(identity)
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(shape.finish().expect("fixed aggregate scalar grammar"));
    KindAspectContractDeclarations::new(vec![DeclaredAspectContractBinding {
        binding: if relation {
            AspectBinding::RelationField {
                field: FieldKey::new(aspect).expect("fixed aggregate binding"),
            }
        } else {
            AspectBinding::EntityField {
                field: FieldKey::new(aspect).expect("fixed aggregate binding"),
            }
        },
        contract,
    }])
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
