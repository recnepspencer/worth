//! End-to-end macro courts for `#[derive(WorthBinaryValue)]`,
//! `#[derive(WorthManagedIndex)]`, and `#[derive(WorthQuery)]` managed-index wiring.
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::{fragment::*, *};
use worth_query_decl::facade::{
    authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_structured_value_binding, WorthManagedIndex, WorthQuery, WorthQueryProjection,
    WorthQueryRecord, WorthQuerySchema,
};

const SOURCE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.managed_dx.source.v1");
const FIND_BOX_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.managed_dx.find_box.v1");
const FIND_TITLE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.managed_dx.find_title.v1");
const ROW_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.managed_dx.row.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "macros.managed_dx",
    name = "ShopSchema",
    members(Shop, ShopRegistration)
)]
pub struct ShopSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, bindings = ShopCodecs, members(Mapping, Principal, Floor, Equipment))]
pub struct Shop;

worth_query_native_scalar_bindings!(pub ShopCodecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for ShopCodecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for ShopCodecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(MappingFacts), relations(MappingTarget))]
pub struct Mapping;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(PrincipalFacts))]
pub struct Principal;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(FloorFacts),
    relations(HasEquipment),
    reference(ShopSchema, Shop)
)]
pub struct Floor;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(EquipmentFacts), reference(ShopSchema, Shop))]
pub struct Equipment;

#[derive(WorthQuerySchema)]
#[worth_query(relation, to = Principal, from = Mapping, reference(ShopSchema, Shop))]
pub struct MappingTarget;
#[derive(WorthQuerySchema)]
#[worth_query(relation, to = Equipment, from = Floor, reference(ShopSchema, Shop))]
pub struct HasEquipment;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = ShopCodecs, aspect = MappingFacts, fragment = Shop, entity = Mapping)]
pub struct MappingRow {
    #[worth_query(equality)]
    pub external_id: WorthQueryExternalPrincipalIdentity,
    #[worth_query(equality)]
    pub status: WorthQueryPrincipalMappingStatus,
}
#[derive(WorthQueryRecord)]
#[worth_query(bindings = ShopCodecs, aspect = PrincipalFacts, fragment = Shop, entity = Principal)]
pub struct PrincipalRow {
    #[worth_query(equality, read_only)]
    pub principal_id: u64,
}
#[derive(WorthQueryRecord)]
#[worth_query(bindings = ShopCodecs, aspect = FloorFacts, fragment = Shop, entity = Floor)]
pub struct FloorRow {
    #[worth_query(column = "FloorId", equality, read_only)]
    pub floor_id: String,
}
#[derive(WorthQueryRecord)]
#[worth_query(bindings = ShopCodecs, aspect = EquipmentFacts, fragment = Shop, entity = Equipment)]
pub struct EquipmentFactsRow {
    #[worth_query(equality)]
    pub equip_key: u64,
    pub title: String,
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
#[worth_query(schema = ShopSchema, root = Equipment, namespace = ROW_NS)]
pub struct EquipmentRow {
    #[worth_query(field = EquipKey)]
    pub key: u64,
    #[worth_query(field = Title)]
    pub title: String,
    #[worth_query(field = MinX)]
    pub min_x: i64,
    #[worth_query(field = MinY)]
    pub min_y: i64,
    #[worth_query(field = MaxX)]
    pub max_x: i64,
    #[worth_query(field = MaxY)]
    pub max_y: i64,
}
worth_query_structured_value_binding!(pub EquipmentRowBinding for EquipmentRow {
    identity: "macros.managed_dx.equipment_row.v1"
});
worth_query_structured_value_binding!(pub ShopDenialBinding for () {
    identity: "macros.managed_dx.denial.v1"
});

pub struct ShopPrincipal;
impl ShopPrincipal {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        ShopSchema,
        Self,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "ShopPrincipal",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalId::at("ExternalId"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    Status::at("Status"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalId::at("PrincipalId"),
                ),
            },
        )
    }
}
pub struct ShopContext;
impl ApplicationBindingContext<ShopSchema> for ShopContext {
    type PrincipalBinding = ShopPrincipal;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = ShopDenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = "macros.managed_dx.idempotency.v1";
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        ShopSchema,
        ShopPrincipal,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ShopPrincipal::reference()
    }
}

use super::managed_index_hard_dx_probes::{QueryBox2D, QueryBox2DBinding};

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = ShopSchema,
    context = ShopContext,
    namespace = SOURCE_NS,
    result = EquipmentRowBinding,
    root = Equipment,
    scope = FloorId,
    limits = 64,
)]
pub struct EquipmentSource {
    #[worth_query(scope)]
    pub floor: String,
}

#[derive(WorthManagedIndex)]
#[worth_query(schema = ShopSchema, entity = Equipment, source = EquipmentSourceBinding, probe = QueryBox2D)]
pub struct EquipmentBoxIndex;

#[derive(WorthManagedIndex)]
#[worth_query(schema = ShopSchema, entity = Equipment, source = EquipmentSourceBinding, probe = String)]
pub struct EquipmentTitleIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = ShopSchema,
    context = ShopContext,
    namespace = FIND_BOX_NS,
    scope = FloorId,
    managed_index = EquipmentBoxIndex,
    lanes = OneShotAndLive,
    limits = 64,
)]
pub struct FindEquipmentInBox {
    #[worth_query(scope)]
    pub floor: String,
    #[worth_query(probe)]
    pub region: QueryBox2D,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = ShopSchema,
    context = ShopContext,
    namespace = FIND_TITLE_NS,
    scope = FloorId,
    managed_index = EquipmentTitleIndex,
    lanes = OneShotAndLive,
    limits = 64,
)]
pub struct FindEquipmentByTitle {
    #[worth_query(scope)]
    pub floor: String,
    #[worth_query(probe)]
    pub prefix: String,
}

fn qbox(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> QueryBox2D {
    QueryBox2D {
        min_x,
        min_y,
        max_x,
        max_y,
    }
}

#[test]
fn derived_binary_value_enforces_version_length_admission_canonical_and_validation() {
    let valid = qbox(-10, -20, 30, 40);
    let encoded = QueryBox2DBinding::encode(&valid).unwrap();
    let ApplicationValue::InlineBytes(ref raw) = encoded else {
        panic!("expected InlineBytes");
    };
    assert_eq!(raw.as_bytes().len(), 33);
    assert_eq!(raw.as_bytes()[0], 1);
    assert_eq!(QueryBox2DBinding::decode(&encoded).unwrap(), valid);

    // Admission refusal propagates cleanly.
    let denied =
        QueryBox2DBinding::decode_admitted(&encoded, &mut |_, _| Err(ValueCodecDenial::Admission));
    assert!(matches!(
        denied,
        Err(ApplicationValueDecodeDenial::Canonical {
            cause: ValueCodecDenial::Admission,
            ..
        })
    ));

    // Version mismatch -> UnsupportedVersion.
    let mut bad_ver = raw.as_bytes().to_vec();
    bad_ver[0] = 9;
    let bad_ver = ApplicationValue::InlineBytes(InlineBytes::try_new(bad_ver).unwrap());
    assert!(matches!(
        QueryBox2DBinding::decode(&bad_ver),
        Err(ApplicationValueDecodeDenial::Canonical {
            cause: ValueCodecDenial::UnsupportedVersion {
                expected: 1,
                observed: 9
            },
            ..
        })
    ));

    // Invalid domain value refused on both encode and decode.
    assert!(matches!(
        QueryBox2DBinding::encode(&qbox(5, 0, 5, 10)),
        Err(ApplicationValueEncodeDenial::Validation(v))
            if v.reason() == "box must have positive area"
    ));
    let mut bad_payload = vec![1u8];
    for n in [5i64, 0, 5, 10] {
        bad_payload.extend_from_slice(&n.to_le_bytes());
    }
    let bad_payload = ApplicationValue::InlineBytes(InlineBytes::try_new(bad_payload).unwrap());
    assert!(matches!(
        QueryBox2DBinding::decode(&bad_payload),
        Err(ApplicationValueDecodeDenial::Validation(v))
            if v.reason() == "box must have positive area"
    ));
}

pub struct ShopRegistration;
impl Registration<ShopSchema> for ShopRegistration {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<ShopSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<ShopSchema>, ApplicationSchemaDeclarationDenial>
    {
        let asc = ApplicationQueryOrderingDirection::Ascending;
        Ok(builder
            .principal_binding(ShopPrincipal::reference())
            .application_query(
                EquipmentSourceQuery::definition()
                    .root_path(
                        ApplicationQueryRootPath::from(Floor::reference())
                            .forward(HasEquipment::reference()),
                    )
                    .order_by(EquipmentRow::key(), asc),
            )
            .application_query_binding::<EquipmentSourceBinding>()
            .managed_index(ApplicationManagedIndexDefinition::<
                ShopSchema,
                EquipmentBoxIndex,
            >::new())
            .managed_index(ApplicationManagedIndexDefinition::<
                ShopSchema,
                EquipmentTitleIndex,
            >::new())
            .application_query(
                FindEquipmentInBoxQuery::definition().order_by(EquipmentRow::key(), asc),
            )
            .application_query_binding::<FindEquipmentInBoxBinding>()
            .application_query(
                FindEquipmentByTitleQuery::definition().order_by(EquipmentRow::key(), asc),
            )
            .application_query_binding::<FindEquipmentByTitleBinding>())
    }
}

#[test]
fn derived_managed_index_and_query_build_valid_definitions_for_binary_and_native_probes() {
    assert_eq!(
        <EquipmentBoxIndex as ApplicationManagedIndexBinding<ShopSchema>>::NAME,
        "equipment_box_index"
    );
    assert_eq!(
        <EquipmentBoxIndex as ApplicationManagedIndexBinding<ShopSchema>>::PROVIDER_VERSION,
        1
    );
    assert_eq!(
        <EquipmentBoxIndex as ApplicationManagedIndexBinding<ShopSchema>>::STRUCTURE,
        ApplicationManagedIndexSourceStructure::BinarySourceTree
    );
    assert_eq!(
        <EquipmentBoxIndex as ApplicationManagedIndexBinding<ShopSchema>>::MEMBERSHIP,
        ApplicationManagedIndexMembership::CheckedSummarySourceTree
    );

    let decl = ShopSchema::declaration().expect("schema with managed indexes must build");
    let members = decl.erased().members();
    let idx_n = members
        .iter()
        .filter(|m| matches!(m, ApplicationSchemaMember::ManagedIndex { .. }))
        .count();
    let q_n = members
        .iter()
        .filter(|m| {
            matches!(
                m,
                ApplicationSchemaMember::ApplicationQuery { definition }
                    if definition.managed_index().is_some()
            )
        })
        .count();
    assert_eq!((idx_n, q_n), (2, 2));

    // Fallible prepare_parameters refuses invalid probe before execution.
    let mk = |r| FindEquipmentInBox {
        floor: "F1".into(),
        region: r,
    };
    assert!(mk(qbox(10, 0, 10, 5)).prepare_parameters().is_err());
    assert!(mk(qbox(0, 0, 10, 10)).prepare_parameters().is_ok());
}
