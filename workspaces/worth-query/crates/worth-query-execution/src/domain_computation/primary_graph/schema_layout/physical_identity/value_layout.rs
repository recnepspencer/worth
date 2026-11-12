//! Pure schema values exercise the same allocation and registration owners.
use super::super::registry_lowering::{
    lower_kind_ids, next_provider_kind_id, register_application_relation, register_entity,
};
use super::*;
use worth_query_declaration::facade::application_schema::*;
mod roles;
use roles::*;
struct Schema;
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.tests.physical.roles";
    const NAME: &'static str = "RoleAllocation";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declaration(false, false, false)
    }
}
fn declaration(
    earlier: bool,
    edge: bool,
    orbit: bool,
) -> Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial> {
    match (earlier, edge, orbit) {
        (false, false, false) => {
            ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(BaseRoles)?.build()
        }
        (true, false, false) => {
            ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(EarlierRoles)?.build()
        }
        (false, true, false) => {
            ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(ExtraEdgeRoles)?.build()
        }
        (true, true, false) => {
            ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(ExtendedRoles)?.build()
        }
        (false, false, true) => {
            ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(OrbitRoles)?.build()
        }
        (true, false, true) => {
            ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(EarlierOrbitRoles)?.build()
        }
        (false, true, true) => {
            ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(ExtraEdgeOrbitRoles)?
                .build()
        }
        (true, true, true) => {
            ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(ExtendedOrbitRoles)?
                .build()
        }
    }
}
pub(super) fn schema(earlier: bool, edge: bool, orbit: bool) -> ErasedApplicationSchemaDeclaration {
    declaration(earlier, edge, orbit).unwrap().into_erased()
}
pub(super) struct ValueLayout {
    pub(super) roles: PhysicalIdentityBindings,
    pub(super) registry: RelationalSchemaRegistry,
}
pub(super) fn lower(
    schema: &ErasedApplicationSchemaDeclaration,
    previous: Option<&PhysicalIdentityBindings>,
) -> ValueLayout {
    let raw = RelationalSchemaRegistry::new();
    let mut roles =
        PhysicalIdentityAllocation::for_identities(previous, &raw, None, std::iter::empty())
            .unwrap();
    let (entities, relations) = lower_kind_ids(schema, &raw, &mut roles).unwrap();
    let (id, version) =
        super::super::registry_lowering::relational_schema_basis(schema, &raw).unwrap();
    let mut registry = RelationalSchemaRegistry::new();
    for (name, kind) in &entities {
        registry = register_entity(registry, &id, version, name, *kind, Vec::new()).unwrap();
    }
    for member in schema.members() {
        if let ApplicationSchemaMember::Relation {
            relation,
            from,
            to,
            integrity,
        } = member
        {
            registry = register_application_relation(
                registry,
                &id,
                version,
                relation,
                relations[relation],
                entities[from],
                entities[to],
                integrity,
                &entities,
                Vec::new(),
            )
            .unwrap();
        }
    }
    let first = roles
        .platform_first(|| {
            next_provider_kind_id(
                &raw,
                entities.values().copied(),
                relations.values().copied(),
            )
        })
        .unwrap();
    let (next, _) = super::super::platform_entity_lowering::lower_platform_entities(
        registry,
        &id,
        version,
        None,
        first,
        entities.values().copied().collect(),
        &mut roles,
    )
    .unwrap();
    registry = next;
    roles.reserve_registry(&registry);
    let (next, _) = super::super::aggregate_identity::lower(
        registry, schema, None, &id, version, &entities, &relations, &mut roles,
    )
    .unwrap();
    let (registry, _) =
        super::super::bootstrap_publication::lower(next, None, &id, version, &mut roles).unwrap();
    let roles = roles.finish(&registry, entities, relations).unwrap();
    ValueLayout { roles, registry }
}
pub(super) fn contracts() -> super::super::registry_lowering::LoweredApplicationContractBindings {
    super::super::registry_lowering::LoweredApplicationContractBindings {
        by_entity: BTreeMap::new(),
        by_relation: BTreeMap::new(),
    }
}

pub(super) fn same_named_classes() -> ErasedApplicationSchemaDeclaration {
    ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(SameNamedRoles)
        .unwrap()
        .build()
        .unwrap()
        .into_erased()
}

pub(super) fn identity_reservation(
    kind: KindId,
    identity: AspectIdentity,
) -> RelationalSchemaRegistry {
    use worth_foundational::facade::{aspects, AspectKey, FieldKey, ScalarAspectType};
    use worth_relational::facade::schema::{AspectBinding, SchemaId, SchemaVersionId};
    let contract = aspects()
        .contract()
        .for_key(AspectKey::new("retired-role").unwrap())
        .identified_by(identity)
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(
            aspects()
                .struct_fields()
                .required("value", ScalarAspectType::UInt64)
                .finish()
                .unwrap(),
        );
    register_entity(
        RelationalSchemaRegistry::new(),
        &SchemaId("historical-role-values".into()),
        SchemaVersionId(1),
        "retired-role",
        kind,
        vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: FieldKey::new("retired-role").unwrap(),
            },
            contract,
        }],
    )
    .unwrap()
}
