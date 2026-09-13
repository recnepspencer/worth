use std::collections::{BTreeMap, BTreeSet};

use worth_query_installation::facade::{
    ApplicationRelationCrossContextPolicy, ApplicationRelationDeletionPolicy,
    ApplicationRelationIntegrity, ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
    WorthQueryInstalledApplicationSchemaContractCatalog,
};
use worth_relational::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{
    CardinalityContractDeclaration, ContractId, DeclaredAspectContractBinding,
    EndpointDeletionIntegrityDeclaration, EndpointDeletionIntegrityMode,
    EndpointKindContractDeclaration, EntityKindRegistration, KindAspectContractDeclarations,
    MinimumCardinalityEnforcement, PairMinimumSemantics, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};

use super::{
    kind_space_exhausted, relational_schema_denial, WorthQueryPrimaryGraphInstallationDenial,
};

pub(super) fn lower_kind_ids(
    schema: &ErasedApplicationSchemaDeclaration,
    existing_registry: &RelationalSchemaRegistry,
) -> Result<
    (BTreeMap<String, KindId>, BTreeMap<String, KindId>),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let mut entity_names = BTreeSet::new();
    let mut relation_names = BTreeSet::new();
    for member in schema.members() {
        match member {
            ApplicationSchemaMember::Entity { entity } => {
                entity_names.insert(entity.clone());
            }
            ApplicationSchemaMember::Relation { relation, .. } => {
                relation_names.insert(relation.clone());
            }
            _ => {}
        }
    }
    let mut next_kind = next_kind_id(existing_registry)?;
    let mut entity_kinds = BTreeMap::new();
    let mut relation_kinds = BTreeMap::new();
    for name in entity_names {
        entity_kinds.insert(name, KindId(next_kind));
        next_kind = next_kind.checked_add(1).ok_or_else(kind_space_exhausted)?;
    }
    for name in relation_names {
        relation_kinds.insert(name, KindId(next_kind));
        next_kind = next_kind.checked_add(1).ok_or_else(kind_space_exhausted)?;
    }
    Ok((entity_kinds, relation_kinds))
}

fn next_kind_id(
    registry: &RelationalSchemaRegistry,
) -> Result<u32, WorthQueryPrimaryGraphInstallationDenial> {
    let authority = registry.authority_snapshot();
    authority
        .entity_kinds
        .iter()
        .map(|kind| kind.kind_id)
        .chain(authority.relation_kinds.iter().map(|kind| kind.kind_id))
        .map(|kind| kind.0)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(kind_space_exhausted)
}

pub(super) fn next_provider_kind_id(
    registry: &RelationalSchemaRegistry,
    entity_kinds: impl Iterator<Item = KindId>,
    relation_kinds: impl Iterator<Item = KindId>,
) -> Result<KindId, WorthQueryPrimaryGraphInstallationDenial> {
    let next_application_kind = entity_kinds
        .chain(relation_kinds)
        .map(|kind| kind.0)
        .max()
        .map(|kind| kind.checked_add(1).ok_or_else(kind_space_exhausted))
        .transpose()?;
    Ok(KindId(
        next_application_kind.unwrap_or(next_kind_id(registry)?),
    ))
}

pub(super) fn relational_schema_basis(
    schema: &ErasedApplicationSchemaDeclaration,
    existing_registry: &RelationalSchemaRegistry,
) -> Result<(SchemaId, SchemaVersionId), WorthQueryPrimaryGraphInstallationDenial> {
    existing_registry
        .authoritative_schema_basis()
        .map_err(relational_schema_denial)
        .map(|basis| {
            basis.unwrap_or_else(|| {
                (
                    SchemaId(format!(
                        "application-schema:{}:{}:{}:{}",
                        schema.owner(),
                        schema.name(),
                        schema.major(),
                        schema.minor(),
                    )),
                    SchemaVersionId(schema.major()),
                )
            })
        })
}

pub(super) fn register_entity(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    schema_version_id: SchemaVersionId,
    entity: &str,
    kind_id: KindId,
    aspects: Vec<DeclaredAspectContractBinding>,
) -> Result<RelationalSchemaRegistry, WorthQueryPrimaryGraphInstallationDenial> {
    registry
        .register_entity_kind(EntityKindRegistration {
            kind_id,
            kind_name: entity.to_string(),
            schema_id: schema_id.clone(),
            schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::new(aspects),
        })
        .map_err(relational_schema_denial)
}

pub(super) fn register_relation(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    schema_version_id: SchemaVersionId,
    relation: &str,
    kind_id: KindId,
    from_kind: KindId,
    to_kind: KindId,
    integrity: ApplicationRelationIntegrity,
) -> Result<RelationalSchemaRegistry, WorthQueryPrimaryGraphInstallationDenial> {
    let cross_context_policy = lower_cross_context(integrity.cross_context_policy());
    let relation_integrity = lower_relation_integrity(relation, from_kind, to_kind, integrity);
    registry
        .register_relation_kind(RelationKindRegistration {
            kind_id,
            kind_name: relation.to_string(),
            schema_id: schema_id.clone(),
            schema_version_id,
            cross_context_policy,
            cascade_delete_policy: lower_cascade_delete(integrity.deletion),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity,
        })
        .map_err(relational_schema_denial)
}

fn lower_cross_context(value: ApplicationRelationCrossContextPolicy) -> CrossContextPolicy {
    match value {
        ApplicationRelationCrossContextPolicy::AllowExplicit => CrossContextPolicy::AllowExplicit,
        ApplicationRelationCrossContextPolicy::SchemaControlled => {
            CrossContextPolicy::SchemaControlled
        }
        ApplicationRelationCrossContextPolicy::Forbid => CrossContextPolicy::Forbid,
    }
}

fn lower_cascade_delete(value: ApplicationRelationDeletionPolicy) -> CascadeDeletePolicy {
    match value {
        ApplicationRelationDeletionPolicy::CascadeDeleteRelations => {
            CascadeDeletePolicy::CascadeDeleteRelations
        }
        _ => CascadeDeletePolicy::RetainDanglingForAudit,
    }
}

fn lower_relation_integrity(
    relation: &str,
    from_kind: KindId,
    to_kind: KindId,
    integrity: ApplicationRelationIntegrity,
) -> RelationIntegrityDeclarations {
    let endpoints = vec![EndpointKindContractDeclaration {
        contract_id: ContractId::new(format!("{relation}:endpoints")),
        allowed_source_kinds: vec![from_kind],
        allowed_target_kinds: vec![to_kind],
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
    RelationIntegrityDeclarations::new(
        endpoints,
        cardinality.into_iter().collect(),
        Vec::new(),
        Vec::new(),
        deletion.into_iter().collect(),
    )
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
mod tests {
    use worth_query_installation::facade::{
        ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
        ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints,
        ApplicationRelationIntegrity,
    };
    use worth_relational::facade::identity::KindId;
    use worth_relational::facade::schema::EndpointDeletionIntegrityMode;

    use super::lower_relation_integrity;

    #[test]
    fn declared_relation_integrity_lowers_exact_endpoint_cardinality_and_deletion_contracts() {
        let declared = ApplicationRelationIntegrity::new(
            ApplicationRelationEndpoints::new(
                true,
                ApplicationRelationCrossContextPolicy::SchemaControlled,
            ),
            ApplicationRelationCardinality::new(
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
            ),
            ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
        );

        let lowered = lower_relation_integrity("Contains", KindId(11), KindId(12), declared);

        let endpoints = &lowered.endpoint_kind_contracts[0];
        assert_eq!(endpoints.allowed_source_kinds, vec![KindId(11)]);
        assert_eq!(endpoints.allowed_target_kinds, vec![KindId(12)]);
        assert!(endpoints.self_edges_allowed);
        let cardinality = &lowered.cardinality_contracts[0];
        assert_eq!(
            (cardinality.source_min, cardinality.source_max),
            (Some(1), Some(2))
        );
        assert_eq!(
            (cardinality.target_min, cardinality.target_max),
            (Some(3), Some(4))
        );
        assert_eq!(
            (cardinality.pair_min, cardinality.pair_max),
            (Some(5), Some(6))
        );
        assert_eq!(
            lowered.endpoint_deletion_integrity_contracts[0].mode,
            EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit,
        );
    }

    #[test]
    fn unbounded_relation_omits_a_false_cardinality_contract() {
        let lowered = lower_relation_integrity(
            "Contains",
            KindId(11),
            KindId(12),
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        );

        assert_eq!(lowered.endpoint_kind_contracts.len(), 1);
        assert!(lowered.endpoint_kind_contracts[0].self_edges_allowed);
        assert!(lowered.cardinality_contracts.is_empty());
        assert!(lowered.endpoint_deletion_integrity_contracts.is_empty());
    }

    #[test]
    fn no_self_edges_uses_the_same_relation_integrity_lowering_path() {
        let lowered = lower_relation_integrity(
            "Contains",
            KindId(11),
            KindId(11),
            ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling(),
        );

        assert_eq!(lowered.endpoint_kind_contracts.len(), 1);
        assert!(!lowered.endpoint_kind_contracts[0].self_edges_allowed);
        assert!(lowered.cardinality_contracts.is_empty());
        assert!(lowered.endpoint_deletion_integrity_contracts.is_empty());
    }
}

pub(super) struct LoweredApplicationContractBindings {
    pub by_entity: BTreeMap<String, Vec<DeclaredAspectContractBinding>>,
}

pub(super) fn lower_application_contract_bindings(
    catalog: &WorthQueryInstalledApplicationSchemaContractCatalog,
) -> LoweredApplicationContractBindings {
    let mut by_entity = BTreeMap::<String, Vec<DeclaredAspectContractBinding>>::new();
    for installed in catalog.contracts() {
        by_entity
            .entry(installed.locus().entity().to_string())
            .or_default()
            .push(DeclaredAspectContractBinding {
                binding: installed.binding().clone(),
                contract: installed.contract().clone(),
            });
    }
    LoweredApplicationContractBindings { by_entity }
}
