//! A storage designation binds only the complete, reserved Native grammar.
use super::*;
use std::collections::BTreeSet;

pub(crate) fn validate_binding(
    registry: &RelationalSchemaRegistry,
    relation: &RelationKindRegistration,
) -> Result<(), SchemaRegistryError> {
    let Some(storage) = &relation.relation_integrity.current_publication_storage else {
        return Ok(());
    };
    let invalid = || {
        SchemaRegistryError::invalid_relation_integrity_declaration(
            relation.kind_id,
            "current publication ownership requires its exact private storage grammar",
        )
    };
    let private = storage
        .metadata_kinds()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let aspects = [
        storage.catalog_aspect,
        storage.publication_aspect,
        storage.edge_aspect,
        storage.link_aspect,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    if registry.current_publication_storage.is_some()
        || private.len() != 7
        || aspects.len() != 4
        || relation.kind_id != storage.scope_link
        || storage.entity_kinds.is_empty()
        || storage
            .entity_kinds
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || storage.entity_kinds.iter().any(|kind| {
            private.contains(kind)
                || registry.native_private_kinds.contains(kind)
                || !registry.entity_kinds.contains_key(kind)
        })
    {
        return Err(invalid());
    }
    for (kind, name, identity) in [
        (storage.catalog_kind, CATALOG, storage.catalog_aspect),
        (
            storage.publication_kind,
            PUBLICATION,
            storage.publication_aspect,
        ),
        (storage.edge_kind, EDGE, storage.edge_aspect),
    ] {
        let actual = registry.entity_kinds.get(&kind).ok_or_else(invalid)?;
        let expected = crate::schema::canonicalize_entity_registration(EntityKindRegistration {
            kind_id: kind,
            kind_name: name.into(),
            schema_id: relation.schema_id.clone(),
            schema_version_id: relation.schema_version_id,
            aspect_contract_declarations: declarations(name, identity, false),
        })?;
        if actual != &expected {
            return Err(invalid());
        }
    }
    for (kind, name, from, to) in [
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
        let actual = if kind == relation.kind_id {
            relation
        } else {
            registry.relation_kinds.get(&kind).ok_or_else(invalid)?
        };
        let mut integrity = endpoints(name, from, to);
        if kind == storage.scope_link {
            integrity = integrity.with_current_publication_storage(storage.clone());
        }
        let expected =
            crate::schema::canonicalize_relation_registration(RelationKindRegistration {
                kind_id: kind,
                kind_name: name.into(),
                schema_id: relation.schema_id.clone(),
                schema_version_id: relation.schema_version_id,
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
                aspect_contract_declarations: declarations(LINK, storage.link_aspect, true),
                relation_integrity: integrity,
            })?;
        if actual != &expected {
            return Err(invalid());
        }
    }
    // Reserved aspect names cannot disguise an ordinary record in historical
    // native metadata-change classification.
    for (kind, declarations) in registry
        .entity_kinds
        .iter()
        .map(|(id, k)| (id, &k.aspect_contract_declarations))
        .chain(
            registry
                .relation_kinds
                .iter()
                .map(|(id, k)| (id, &k.aspect_contract_declarations)),
        )
    {
        if !private.contains(kind)
            && declarations.aspects.iter().any(|aspect| {
                [CATALOG, PUBLICATION, EDGE, LINK].contains(&aspect.contract.key().as_str())
            })
        {
            return Err(invalid());
        }
    }
    Ok(())
}
