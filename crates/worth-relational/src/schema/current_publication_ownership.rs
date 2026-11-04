//! Conditional registration of ordinary region storage for current ownership.
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::identity::data::KindId;
use crate::schema::data::{
    AspectBinding, ContractId, CurrentPublicationStorage, DeclaredAspectContractBinding,
    EndpointKindContractDeclaration, EntityKindRegistration, KindAspectContractDeclarations,
    RelationIntegrityDeclarations, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
    SchemaRegistryError, SchemaVersionId,
};
use worth_foundational::facade::{aspects, AspectIdentity, AspectKey, FieldKey, ScalarAspectType};

mod validation;
pub(crate) use validation::validate_binding;

pub(crate) const CATALOG: &str = "worth-current-publication-catalog";
pub(crate) const PUBLICATION: &str = "worth-current-publication";
pub(crate) const EDGE: &str = "worth-current-publication-edge";
pub(crate) const LINK: &str = "worth-current-publication-link";

/// Platform installation, conditional on an actual source-scope-owned producer.
/// Ordinary mutations receive no grant to these private records.
pub fn register_current_publication_storage(
    registry: RelationalSchemaRegistry,
    schema: &SchemaId,
    version: SchemaVersionId,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    if registry.current_publication_storage.is_some() {
        return Ok(registry);
    }
    let exhausted = || {
        SchemaRegistryError::inconsistent_schema_basis(
            "current publication storage identity space exhausted",
        )
    };
    let mut next = registry
        .entity_kinds
        .keys()
        .chain(registry.relation_kinds.keys())
        .map(|kind| kind.0)
        .max()
        .unwrap_or(0);
    let mut kind = || {
        next = next.checked_add(1).ok_or_else(exhausted)?;
        Ok(KindId(next))
    };
    let kinds: [KindId; 7] = [
        kind()?,
        kind()?,
        kind()?,
        kind()?,
        kind()?,
        kind()?,
        kind()?,
    ];
    let mut aspect = registry
        .entity_kinds
        .values()
        .map(|kind| &kind.aspect_contract_declarations)
        .chain(
            registry
                .relation_kinds
                .values()
                .map(|kind| &kind.aspect_contract_declarations),
        )
        .flat_map(|declarations| &declarations.aspects)
        .map(|aspect| aspect.contract.identity().0)
        .max()
        .unwrap_or(0);
    let mut identity = || {
        aspect = aspect.checked_add(1).ok_or_else(exhausted)?;
        Ok(AspectIdentity(aspect))
    };
    let storage = CurrentPublicationStorage {
        catalog_kind: kinds[0],
        publication_kind: kinds[1],
        edge_kind: kinds[2],
        publication_link: kinds[3],
        member_link: kinds[4],
        edge_link: kinds[5],
        scope_link: kinds[6],
        catalog_aspect: identity()?,
        publication_aspect: identity()?,
        edge_aspect: identity()?,
        link_aspect: identity()?,
        entity_kinds: registry
            .entity_kinds
            .keys()
            .copied()
            .filter(|kind| !registry.native_private_kinds.contains(kind))
            .collect(),
    };
    let mut registry = registry;
    for (kind_id, name, identity) in [
        (storage.catalog_kind, CATALOG, storage.catalog_aspect),
        (
            storage.publication_kind,
            PUBLICATION,
            storage.publication_aspect,
        ),
        (storage.edge_kind, EDGE, storage.edge_aspect),
    ] {
        registry = registry.register_entity_kind(EntityKindRegistration {
            kind_id,
            kind_name: name.into(),
            schema_id: schema.clone(),
            schema_version_id: version,
            aspect_contract_declarations: declarations(name, identity, false),
        })?;
    }
    for (kind_id, name, from, to) in [
        (
            storage.publication_link,
            "worth-current-publication-catalog-link",
            vec![storage.publication_kind],
            vec![storage.catalog_kind],
        ),
        (
            storage.member_link,
            "worth-current-publication-member",
            storage.entity_kinds.clone(),
            vec![storage.publication_kind],
        ),
        (
            storage.edge_link,
            "worth-current-publication-edge-link",
            vec![storage.edge_kind],
            vec![storage.publication_kind],
        ),
        (
            storage.scope_link,
            "worth-current-publication-scope",
            vec![storage.catalog_kind],
            storage.entity_kinds.clone(),
        ),
    ] {
        let mut integrity = endpoints(name, from, to);
        if kind_id == storage.scope_link {
            integrity = integrity.with_current_publication_storage(storage.clone());
        }
        registry = registry.register_relation_kind(RelationKindRegistration {
            kind_id,
            kind_name: name.into(),
            schema_id: schema.clone(),
            schema_version_id: version,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: declarations(LINK, storage.link_aspect, true),
            relation_integrity: integrity,
        })?;
    }
    Ok(registry)
}

pub(super) fn declarations(
    name: &str,
    identity: AspectIdentity,
    relation: bool,
) -> KindAspectContractDeclarations {
    let contract = aspects()
        .contract()
        .for_key(AspectKey::new(name).expect("fixed roster aspect"))
        .identified_by(identity)
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(
            aspects()
                .struct_fields()
                .required("publication", ScalarAspectType::UInt64)
                .required("payload", ScalarAspectType::String)
                .finish()
                .expect("fixed roster fields"),
        );
    let field = FieldKey::new(name).expect("fixed roster binding");
    KindAspectContractDeclarations::new(vec![DeclaredAspectContractBinding {
        binding: if relation {
            AspectBinding::RelationField { field }
        } else {
            AspectBinding::EntityField { field }
        },
        contract,
    }])
}

pub(super) fn endpoints(
    name: &str,
    from: Vec<KindId>,
    to: Vec<KindId>,
) -> RelationIntegrityDeclarations {
    RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: ContractId::new(format!("{name}:endpoints")),
            allowed_source_kinds: from,
            allowed_target_kinds: to,
            self_edges_allowed: false,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
        }],
        vec![],
        vec![],
        vec![],
        vec![],
    )
}
