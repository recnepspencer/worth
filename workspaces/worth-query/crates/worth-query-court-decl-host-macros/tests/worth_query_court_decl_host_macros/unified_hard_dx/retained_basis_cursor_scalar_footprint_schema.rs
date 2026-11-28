//! Minimal custom heap-backed scalar schema & program for
//! `retained_basis_cursor_scalar_footprint_court`.

use worth_foundational::facade::{AspectValue, InlineBytes, InternedString, ScalarAspectType};
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::*;
use worth_query_host::facade::historical_cursor::KnownProjectedHeapBytes;
use worth_query_host::facade::worth_query_application_program;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpandedTagScalar(pub String);

impl KnownProjectedHeapBytes for ExpandedTagScalar {
    fn known_heap_bytes(&self) -> usize {
        self.0.capacity()
    }
}

pub struct ExpandedTagBinding;

impl ApplicationScalarValueBinding for ExpandedTagBinding {
    type Value = ExpandedTagScalar;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.custom-scalar-budget.expanded-tag.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::String(InternedString::Raw(value.0.clone())))
    }
}

impl ApplicationReadableScalarValueBinding for ExpandedTagBinding {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        match value {
            AspectValue::String(InternedString::Raw(s)) => {
                Ok(ExpandedTagScalar(format!("{s}-{}", "H".repeat(4096))))
            }
            _ => Err(ApplicationValueDecodeDenial::CodecRejected {
                binding_identity:
                    worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(
                        Self::IDENTITY_NAME,
                    ),
            }),
        }
    }
}

/// Scalar decoder that returns an `AspectValue::InlineBytes` with a 1-byte
/// payload backed by an over-allocated `Vec::with_capacity(8192)`.
pub struct OverAllocatedInlineBytesBinding;

impl ApplicationScalarValueBinding for OverAllocatedInlineBytesBinding {
    type Value = AspectValue;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.custom-scalar-budget.overalloc-bytes.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::InlineBytes;
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(value.clone())
    }
}

impl ApplicationReadableScalarValueBinding for OverAllocatedInlineBytesBinding {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        match value {
            AspectValue::InlineBytes(bytes) => {
                let mut overalloc = Vec::with_capacity(8192);
                overalloc.extend_from_slice(bytes.as_bytes());
                let wrapped = InlineBytes::try_new(overalloc).expect("inline bytes within limit");
                Ok(AspectValue::InlineBytes(wrapped))
            }
            _ => Err(ApplicationValueDecodeDenial::CodecRejected {
                binding_identity:
                    worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(
                        Self::IDENTITY_NAME,
                    ),
            }),
        }
    }
}

worth_query_principal_issuer!(pub ScalarBudgetIssuer);
worth_query_authentication_audience!(pub ScalarBudgetAudience);
worth_query_authentication_method!(pub ScalarBudgetMethod);

const SCALAR_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("worth.custom-scalar-budget.ns.v1");
const OVERALLOC_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("worth.custom-scalar-budget.overalloc-ns.v1");

#[derive(Clone, Copy)]
pub struct ScalarSchema;
impl Fragment for ScalarSchema {
    const NAME: &'static str = "custom_scalar_budget";
    type Bindings = NoBindings;
    type Members = (ScalarContribution,);
}
impl ApplicationSchema for ScalarSchema {
    const OWNER: &'static str = "worth.query.custom-scalar-budget";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for ScalarSchema {
    type Contributions = <Self as Fragment>::Members;
}
#[derive(Clone, Copy)]
pub struct ScalarFrag;
impl Fragment for ScalarFrag {
    const NAME: &'static str = "scalar_frag";
    type Bindings = ScalarBindings;
    type Members = (SExtMap, SPrin, SBox, SItem);
}
pub struct ScalarBindings;
impl BindingSet for ScalarBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        w: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(w)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for ScalarBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for ScalarBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl Scalar<u64> for ScalarBindings {
    type Binding = U64ApplicationValueBinding;
}
impl Scalar<ExpandedTagScalar> for ScalarBindings {
    type Binding = ExpandedTagBinding;
}
impl Scalar<AspectValue> for ScalarBindings {
    type Binding = OverAllocatedInlineBytesBinding;
}

macro_rules! ent {
    ($n:ident, $a:ty, $r:ty) => {
        #[derive(Clone, Copy)]
        pub struct $n;
        impl Entity for $n {
            const NAME: &'static str = stringify!($n);
            type Aspects = $a;
            type Relations = $r;
        }
        impl $n {
            pub fn reference() -> ApplicationEntityRef<ScalarSchema, Self> {
                entity(ScalarFrag, Self)
            }
        }
    };
}
ent!(SExtMap, (SExtId,), (SMapTarget,));
ent!(SPrin, (SPrinFacts,), ());
ent!(SBox, (SBoxFacts,), (SContains,));
ent!(SItem, (SItemFacts,), ());

pub struct SExtId;
pub type SExtRow = (
    WorthQueryExternalPrincipalIdentity,
    WorthQueryPrincipalMappingStatus,
);
impl Aspect for SExtId {
    const NAME: &'static str = "SExtId";
    type Row = SExtRow;
}
impl Record<SExtRow> for ScalarBindings {
    fn record() -> impl Shape<SExtRow, Self> {
        field("SExtIdField", |r: &SExtRow| r.0.clone())
            .equality_queryable(true)
            .and("SMapStatus", |r: &SExtRow| r.1)
            .equality_queryable(true)
            .build(|a, b, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>((a, b))
            })
    }
}
pub struct SPrinFacts;
pub struct SPrinRow(pub u64);
impl Aspect for SPrinFacts {
    const NAME: &'static str = "SPrinFacts";
    type Row = SPrinRow;
}
impl Record<SPrinRow> for ScalarBindings {
    fn record() -> impl Shape<SPrinRow, Self> {
        field("SPrinId", |r: &SPrinRow| r.0)
            .equality_queryable(true)
            .read_only()
            .build(|a, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SPrinRow(a))
            })
    }
}
pub struct SBoxFacts;
pub struct SBoxRow(pub u64);
impl Aspect for SBoxFacts {
    const NAME: &'static str = "SBoxFacts";
    type Row = SBoxRow;
}
impl Record<SBoxRow> for ScalarBindings {
    fn record() -> impl Shape<SBoxRow, Self> {
        field("SBoxKey", |r: &SBoxRow| r.0)
            .equality_queryable(true)
            .build(|a, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SBoxRow(a))
            })
    }
}
pub struct SItemFacts;
pub struct SItemRow(pub u64, pub ExpandedTagScalar, pub AspectValue);
impl Aspect for SItemFacts {
    const NAME: &'static str = "SItemFacts";
    type Row = SItemRow;
}
impl Record<SItemRow> for ScalarBindings {
    fn record() -> impl Shape<SItemRow, Self> {
        field("SItemKey", |r: &SItemRow| r.0)
            .equality_queryable(true)
            .and("SItemTag", |r: &SItemRow| r.1.clone())
            .and("SItemBlob", |r: &SItemRow| r.2.clone())
            .build(|a, b, c, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SItemRow(a, b, c))
            })
    }
}
macro_rules! rel {
    ($n:ident, $f:ident -> $t:ident) => {
        pub struct $n;
        impl Relation for $n {
            const NAME: &'static str = stringify!($n);
            type To = $t;
        }
        impl $n {
            pub fn reference() -> ApplicationRelationRef<ScalarSchema, Self, $f, $t> {
                relation(ScalarFrag, $f, Self)
            }
        }
    };
}
rel!(SMapTarget, SExtMap -> SPrin);
rel!(SContains, SBox -> SItem);

pub type SExtIdField =
    FieldSelection<ScalarFrag, SExtMap, SExtId, WorthQueryExternalPrincipalIdentity>;
pub type SMapStatus = FieldSelection<ScalarFrag, SExtMap, SExtId, WorthQueryPrincipalMappingStatus>;
pub type SPrinId = FieldSelection<ScalarFrag, SPrin, SPrinFacts, u64>;
pub type SBoxKey = FieldSelection<ScalarFrag, SBox, SBoxFacts, u64>;
pub type SItemKey = FieldSelection<ScalarFrag, SItem, SItemFacts, u64>;
pub type SItemTag = FieldSelection<ScalarFrag, SItem, SItemFacts, ExpandedTagScalar>;
pub type SItemBlob = FieldSelection<ScalarFrag, SItem, SItemFacts, AspectValue>;

pub struct ScalarPrincipal;
impl ScalarPrincipal {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        ScalarSchema,
        Self,
        SExtMap,
        SPrin,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "ScalarPrincipal",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    SExtIdField::at("SExtIdField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    SMapStatus::at("SMapStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    SMapTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    SPrinId::at("SPrinId"),
                ),
            },
        )
    }
}
impl Registration<ScalarSchema> for ScalarPrincipal {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<ScalarSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<ScalarSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.principal_binding(Self::reference()))
    }
}

pub struct ScalarCtx;
impl ApplicationBindingContext<ScalarSchema> for ScalarCtx {
    type PrincipalBinding = ScalarPrincipal;
    type Mapping = SExtMap;
    type Principal = SPrin;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = UnitApplicationValueBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = worth_query_identity!(SCALAR_NS, idempotency);
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        ScalarSchema,
        ScalarPrincipal,
        SExtMap,
        SPrin,
        u64,
        U64ApplicationValueBinding,
    > {
        ScalarPrincipal::reference()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = ScalarSchema, root = SItem, namespace = SCALAR_NS)]
pub struct CustomScalarItemRow {
    #[worth_query(field = SItemKey)]
    pub item_key: u64,
    #[worth_query(field = SItemTag)]
    pub tag: ExpandedTagScalar,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = ScalarSchema,
    context = ScalarCtx,
    namespace = SCALAR_NS,
    result = CustomScalarItemRow,
    root = SItem,
    scope = SBoxKey,
    root_path(SContains),
    order_by(CustomScalarItemRow::item_key(), Ascending),
    limits = 8,
)]
pub struct ReadCustomScalarItems {
    #[worth_query(scope)]
    pub box_key: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = ScalarSchema, root = SItem, namespace = OVERALLOC_NS)]
pub struct OverAllocatedBytesItemRow {
    #[worth_query(field = SItemKey)]
    pub item_key: u64,
    #[worth_query(field = SItemBlob)]
    pub blob: AspectValue,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = ScalarSchema,
    context = ScalarCtx,
    namespace = OVERALLOC_NS,
    result = OverAllocatedBytesItemRow,
    root = SItem,
    scope = SBoxKey,
    root_path(SContains),
    order_by(OverAllocatedBytesItemRow::item_key(), Ascending),
    limits = 8,
)]
pub struct ReadOverAllocatedBytesItems {
    #[worth_query(scope)]
    pub box_key: u64,
}

worth_query_application_program! {
    pub program ScalarProgram for ScalarSchema {
        identity: "worth.custom-scalar-budget.program.v1",
        contribution: ScalarContribution {
            identity: "worth.custom-scalar-budget.contribution.v1",
            members: [ScalarFrag, ScalarPrincipal],
        },
        feature: ScalarFeature => ScalarDef / ScalarRuntime {
            identity: "worth.custom-scalar-budget.feature.v1",
            query(ReadCustomScalarItemsBinding) {
                definition: ReadCustomScalarItemsQuery::definition();
            };
            query(ReadOverAllocatedBytesItemsBinding) {
                definition: ReadOverAllocatedBytesItemsQuery::definition();
            };
        }
    }
}
