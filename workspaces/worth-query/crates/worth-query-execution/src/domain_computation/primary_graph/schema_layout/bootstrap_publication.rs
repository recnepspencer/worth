//! Bootstrap completion is ordinary Query-owned Native data at every location.
use super::*;
use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType};
use worth_relational::facade::schema::{AspectBinding, SchemaId, SchemaVersionId};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct BootstrapPublicationLayout {
    pub kind: KindId,
    pub fields: [AspectFieldLocator; 8],
}

pub(super) fn lower(
    registry: RelationalSchemaRegistry,
    maximum_application_identity: Option<AspectIdentity>,
    schema: &SchemaId,
    version: SchemaVersionId,
    roles: &mut super::physical_identity::PhysicalIdentityAllocation<'_>,
) -> Result<
    (RelationalSchemaRegistry, BootstrapPublicationLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let (kind, identity) = roles.bootstrap(|| {
        let authority = registry.authority_snapshot();
        let kind = authority
            .entity_kinds
            .iter()
            .map(|entry| entry.kind_id)
            .chain(authority.relation_kinds.iter().map(|entry| entry.kind_id))
            .map(|kind| kind.0)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .map(KindId)
            .ok_or_else(kind_space_exhausted)?;
        // Preserve all prior reservations even when this schema has no aggregate.
        let reserved = platform_identity_allocator::allocate_projected_aggregate_aspect_identities(
            maximum_application_identity,
        )?;
        let identity = AspectIdentity(
            reserved[1]
                .0
                .checked_add(1)
                .ok_or_else(contract_space_exhausted)?,
        );
        Ok((kind, identity))
    })?;
    let names = [
        "format-version",
        "binding",
        "terminal-bootstrap-commit",
        "principal-bindings",
        "identity-indexes",
        "application-equality-indexes",
        "policy-entities",
        "policy-relations",
    ];
    let shape = aspects()
        .struct_fields()
        .required("format-version", ScalarAspectType::UInt64)
        .required("binding", ScalarAspectType::String)
        .required("terminal-bootstrap-commit", ScalarAspectType::UInt64)
        .required("principal-bindings", ScalarAspectType::UInt64)
        .required("identity-indexes", ScalarAspectType::UInt64)
        .required("application-equality-indexes", ScalarAspectType::UInt64)
        .required("policy-entities", ScalarAspectType::UInt64)
        .required("policy-relations", ScalarAspectType::UInt64);
    let contract = aspects()
        .contract()
        .for_key(valid_aspect_key("bootstrap-publication")?)
        .identified_by(identity)
        .at_revision(aspects().vocabulary().revision(2))
        .struct_aspect(
            shape
                .finish()
                .map_err(|_| invalid_member("bootstrap-publication"))?,
        );
    let registry = register_entity(
        registry,
        schema,
        version,
        "worth-query-bootstrap-publication",
        kind,
        vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: valid_field_key("bootstrap-publication")?,
            },
            contract,
        }],
    )?;
    let fields = names
        .map(|name| planned_field_locator("bootstrap-publication", name))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .expect("eight completion fields");
    Ok((registry, BootstrapPublicationLayout { kind, fields }))
}

pub(super) fn readmit_role(
    registry: &RelationalSchemaRegistry,
    snapshot: &worth_relational::facade::schema::SchemaAuthoritySnapshot,
    application: &std::collections::BTreeSet<KindId>,
) -> Result<(KindId, AspectIdentity), WorthQueryPrimaryGraphInstallationDenial> {
    super::physical_identity::private_entity_role(
        registry,
        snapshot,
        application,
        "worth-query-bootstrap-publication",
        "bootstrap-publication",
    )
}
