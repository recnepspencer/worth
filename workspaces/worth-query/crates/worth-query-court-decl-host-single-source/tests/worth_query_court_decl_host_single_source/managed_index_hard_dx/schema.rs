//! Hard DX schema: Daymark TextPrefix + Interval1D and Shop Floor Box2D + Box3D + FloatBox2D
//! declared with `WorthBinaryValue`, `WorthManagedIndex`, and `WorthQuery`.
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::{fragment::*, *};
use worth_query_decl::facade::{
    authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_structured_value_binding, WorthBinaryValue, WorthManagedIndex, WorthQuery,
    WorthQueryProjection, WorthQueryRecord, WorthQuerySchema,
};
use worth_query_host::facade::ordered_index::{
    WorthQueryBox2DF64, WorthQueryBox2DProbe, WorthQueryBox3DProbe, WorthQueryInterval1DProbe,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;

pub const ROW_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("hard_dx.shop.row.v1");
pub const SRC_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("hard_dx.shop.source.v1");
pub const PREFIX_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("hard_dx.shop.find_prefix.v1");
pub const INTERVAL_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("hard_dx.shop.find_interval.v1");
pub const BOX2D_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("hard_dx.shop.find_box2d.v1");
pub const BOX3D_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("hard_dx.shop.find_box3d.v1");
pub const FBOX2D_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("hard_dx.shop.find_fbox2d.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.tests.hard_dx",
    name = "ShopFloorSchema",
    members(super::Contribution)
)]
pub struct ShopFloorSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, bindings = ShopCodecs, members(ExternalMapping, Principal, Floor, Asset))]
pub struct ShopFloor;

worth_query_native_scalar_bindings!(pub ShopCodecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for ShopCodecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for ShopCodecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(ExternalIdentity), relations(MappingTarget))]
pub struct ExternalMapping;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(PrincipalFacts))]
pub struct Principal;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(FloorFacts),
    relations(ContainsAsset),
    reference(ShopFloorSchema, ShopFloor)
)]
pub struct Floor;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(AssetFacts), reference(ShopFloorSchema, ShopFloor))]
pub struct Asset;

#[derive(WorthQuerySchema)]
#[worth_query(relation, to = Principal, from = ExternalMapping, reference(ShopFloorSchema, ShopFloor))]
pub struct MappingTarget;
#[derive(WorthQuerySchema)]
#[worth_query(relation, to = Asset, from = Floor, reference(ShopFloorSchema, ShopFloor))]
pub struct ContainsAsset;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = ShopCodecs, aspect = ExternalIdentity, fragment = ShopFloor, entity = ExternalMapping)]
pub struct ExternalIdentityRow {
    #[worth_query(equality)]
    pub external_id: WorthQueryExternalPrincipalIdentity,
    #[worth_query(equality)]
    pub status: WorthQueryPrincipalMappingStatus,
}
#[derive(WorthQueryRecord)]
#[worth_query(bindings = ShopCodecs, aspect = PrincipalFacts, fragment = ShopFloor, entity = Principal)]
pub struct PrincipalFactsRow {
    #[worth_query(equality, read_only)]
    pub principal_id: u64,
}
#[derive(WorthQueryRecord)]
#[worth_query(bindings = ShopCodecs, aspect = FloorFacts, fragment = ShopFloor, entity = Floor)]
pub struct FloorFactsRow {
    #[worth_query(column = "FloorId", equality, read_only)]
    pub floor_id: String,
}
#[derive(WorthQueryRecord)]
#[worth_query(bindings = ShopCodecs, aspect = AssetFacts, fragment = ShopFloor, entity = Asset)]
pub struct AssetFactsRow {
    #[worth_query(equality)]
    pub asset_key: u64,
    pub title: String,
    pub start_min: i64,
    pub end_min: i64,
    pub min_x: i64,
    pub min_y: i64,
    pub min_z: i64,
    pub max_x: i64,
    pub max_y: i64,
    pub max_z: i64,
    pub fmin_x: CanonicalF64,
    pub fmin_y: CanonicalF64,
    pub fmax_x: CanonicalF64,
    pub fmax_y: CanonicalF64,
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
#[worth_query(schema = ShopFloorSchema, root = Asset, namespace = ROW_NS)]
pub struct AssetRow {
    #[worth_query(field = AssetKey)]
    pub key: u64,
    #[worth_query(field = Title)]
    pub title: String,
    #[worth_query(field = StartMin)]
    pub start_min: i64,
    #[worth_query(field = EndMin)]
    pub end_min: i64,
    #[worth_query(field = MinX)]
    pub min_x: i64,
    #[worth_query(field = MinY)]
    pub min_y: i64,
    #[worth_query(field = MinZ)]
    pub min_z: i64,
    #[worth_query(field = MaxX)]
    pub max_x: i64,
    #[worth_query(field = MaxY)]
    pub max_y: i64,
    #[worth_query(field = MaxZ)]
    pub max_z: i64,
    #[worth_query(field = FminX)]
    pub fmin_x: CanonicalF64,
    #[worth_query(field = FminY)]
    pub fmin_y: CanonicalF64,
    #[worth_query(field = FmaxX)]
    pub fmax_x: CanonicalF64,
    #[worth_query(field = FmaxY)]
    pub fmax_y: CanonicalF64,
}
worth_query_structured_value_binding!(pub AssetRowBinding for AssetRow { identity: "hard_dx.asset_row.v1" });
worth_query_structured_value_binding!(pub BodyResultBinding for () { identity: "hard_dx.body_ok.v1" });
worth_query_structured_value_binding!(pub BodyDenialBinding for WorthQueryOperationBodyDenial { identity: "hard_dx.body_denial.v1" });

pub struct PrincipalBinding;
impl PrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        ShopFloorSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "PrincipalBinding",
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
impl Registration<ShopFloorSchema> for PrincipalBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<ShopFloorSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<ShopFloorSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}
pub struct ShopContext;
impl ApplicationBindingContext<ShopFloorSchema> for ShopContext {
    type PrincipalBinding = PrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = BodyDenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = "hard_dx.idempotency.v1";
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        ShopFloorSchema,
        PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PrincipalBinding::reference()
    }
}

// =============================================================================
// Hard DX Consumer Declarations (~15-20 LOC per index family!)
// =============================================================================

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(schema = ShopFloorSchema, context = ShopContext, namespace = SRC_NS,
    result = AssetRowBinding, root = Asset, scope = FloorId, limits = 64)]
pub struct AssetSource {
    #[worth_query(scope)]
    pub floor: String,
}

// 1. Daymark Title-Prefix Palette (native String probe)
#[derive(WorthManagedIndex)]
#[worth_index(
    schema = ShopFloorSchema,
    record = Asset,
    source = AssetSourceBinding,
    probe = String,
    kind = text_prefix(AssetRow::title()),
    namespace = "daymark.title_prefix",
    version = 1,
)]
pub struct TitlePrefixIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(schema = ShopFloorSchema, context = ShopContext, namespace = PREFIX_NS,
    scope = FloorId, managed_index = TitlePrefixIndex, lanes = OneShotAndLive, limits = 64)]
pub struct FindByTitlePrefix {
    #[worth_query(scope)]
    pub floor: String,
    #[worth_query(probe)]
    pub prefix: String,
}

// 2. Daymark Calendar Interval (derived binary FocusWindow probe)
fn check_window(w: &FocusWindow) -> Result<(), &'static str> {
    (w.start < w.end).then_some(()).ok_or("start must be < end")
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
#[worth_binary(namespace = "daymark.focus_window", version = 1, validate = check_window)]
pub struct FocusWindow {
    pub start: i64,
    pub end: i64,
}
impl WorthQueryInterval1DProbe<i64> for FocusWindow {
    fn interval_1d_bounds(&self) -> (i64, i64) {
        (self.start, self.end)
    }
}
#[derive(WorthManagedIndex)]
#[worth_index(
    schema = ShopFloorSchema,
    record = Asset,
    source = AssetSourceBinding,
    probe = FocusWindow,
    kind = interval_1d(AssetRow::start_min(), AssetRow::end_min()),
    namespace = "daymark.focus_overlap",
    version = 1,
)]
pub struct FocusIntervalIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(schema = ShopFloorSchema, context = ShopContext, namespace = INTERVAL_NS,
    scope = FloorId, managed_index = FocusIntervalIndex, lanes = OneShotAndLive, limits = 64)]
pub struct FindByFocusWindow {
    #[worth_query(scope)]
    pub floor: String,
    #[worth_query(probe)]
    pub window: FocusWindow,
}

// 3. Shop Floor Equipment Box2D (half-open i64)
fn check_box2d(b: &QueryBox2D) -> Result<(), &'static str> {
    (b.min_x < b.max_x && b.min_y < b.max_y)
        .then_some(())
        .ok_or("2D box must be non-empty")
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
#[worth_binary(namespace = "shopfloor.query_box2d", version = 1, validate = check_box2d)]
pub struct QueryBox2D {
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}
impl WorthQueryBox2DProbe<i64> for QueryBox2D {
    fn box_2d_bounds(&self) -> ([i64; 2], [i64; 2]) {
        ([self.min_x, self.min_y], [self.max_x, self.max_y])
    }
}
#[derive(WorthManagedIndex)]
#[worth_index(
    schema = ShopFloorSchema,
    record = Asset,
    source = AssetSourceBinding,
    probe = QueryBox2D,
    kind = box_2d(AssetRow::min_x(), AssetRow::min_y(), AssetRow::max_x(), AssetRow::max_y()),
    namespace = "shopfloor.equipment_box2d",
    version = 1,
)]
pub struct EquipmentBox2DIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(schema = ShopFloorSchema, context = ShopContext, namespace = BOX2D_NS,
    scope = FloorId, managed_index = EquipmentBox2DIndex, lanes = OneShotAndLive, limits = 64)]
pub struct FindEquipmentInBox2D {
    #[worth_query(scope)]
    pub floor: String,
    #[worth_query(probe)]
    pub region: QueryBox2D,
}

// 4. Shop Floor Route Box3D (closed i64)
fn check_box3d(b: &QueryBox3D) -> Result<(), &'static str> {
    (b.min_x <= b.max_x && b.min_y <= b.max_y && b.min_z <= b.max_z)
        .then_some(())
        .ok_or("3D box must have min <= max")
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
#[worth_binary(namespace = "shopfloor.query_box3d", version = 1, validate = check_box3d)]
pub struct QueryBox3D {
    pub min_x: i64,
    pub min_y: i64,
    pub min_z: i64,
    pub max_x: i64,
    pub max_y: i64,
    pub max_z: i64,
}
impl WorthQueryBox3DProbe<i64> for QueryBox3D {
    fn box_3d_bounds(&self) -> ([i64; 3], [i64; 3]) {
        (
            [self.min_x, self.min_y, self.min_z],
            [self.max_x, self.max_y, self.max_z],
        )
    }
}
#[derive(WorthManagedIndex)]
#[worth_index(
    schema = ShopFloorSchema,
    record = Asset,
    source = AssetSourceBinding,
    probe = QueryBox3D,
    kind = box_3d_closed(
        (AssetRow::min_x(), AssetRow::min_y(), AssetRow::min_z()),
        (AssetRow::max_x(), AssetRow::max_y(), AssetRow::max_z()),
    ),
    namespace = "shopfloor.route_box3d",
    version = 1,
)]
pub struct RouteBox3DIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(schema = ShopFloorSchema, context = ShopContext, namespace = BOX3D_NS,
    scope = FloorId, managed_index = RouteBox3DIndex, lanes = OneShotAndLive, limits = 64)]
pub struct FindRoutesInBox3D {
    #[worth_query(scope)]
    pub floor: String,
    #[worth_query(probe)]
    pub region: QueryBox3D,
}

// 5. Shop Floor CanonicalF64 Box2D (using stock WorthQueryBox2DF64 probe!)
#[derive(WorthManagedIndex)]
#[worth_index(
    schema = ShopFloorSchema,
    record = Asset,
    source = AssetSourceBinding,
    probe = WorthQueryBox2DF64,
    kind = box_2d(AssetRow::fmin_x(), AssetRow::fmin_y(), AssetRow::fmax_x(), AssetRow::fmax_y()),
    namespace = "shopfloor.float_box2d",
    version = 1,
)]
pub struct FloatBox2DIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(schema = ShopFloorSchema, context = ShopContext, namespace = FBOX2D_NS,
    scope = FloorId, managed_index = FloatBox2DIndex, lanes = OneShotAndLive, limits = 64)]
pub struct FindInFloatBox2D {
    #[worth_query(scope)]
    pub floor: String,
    #[worth_query(probe)]
    pub region: WorthQueryBox2DF64,
}
