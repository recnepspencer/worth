//! Minimal schemas for self-referential managed-index Source occurrence courts.
use worth_query_declaration::facade::{
    application_query::*,
    application_schema::{fragment::*, *},
    authentication::*,
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_portable_type, worth_query_query_binding,
    worth_query_structured_value_binding, worth_query_value_binding,
};

pub(super) struct SelfRefSchema;
impl Fragment for SelfRefSchema {
    const NAME: &'static str = "SelfRefSchema";
    type Bindings = Bindings;
    type Members = (
        ExternalMapping,
        Principal,
        Scope,
        ZeroNode,
        FieldedNode,
        Tag,
    );
}
impl ApplicationSchema for SelfRefSchema {
    const OWNER: &'static str = "worth.tests.managed_source.self_ref";
    const NAME: &'static str = "SelfRefSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::from_fragment(Self)?
            .principal_binding(PrincipalBinding::reference())
            .application_query(zero_source_query())
            .application_query_binding::<ZeroSourceBinding>()
            .managed_index(ApplicationManagedIndexDefinition::<Self, ZeroFieldIndex>::new())
            .application_query(fielded_source_query())
            .application_query_binding::<FieldedSourceBinding>()
            .managed_index(ApplicationManagedIndexDefinition::<Self, FieldedIndex>::new())
            .build()
    }
}

pub(super) struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for Bindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Bindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
impl Scalar<String> for Bindings {
    type Binding = StringApplicationValueBinding;
}

macro_rules! entity_decl {
    ($name:ident, aspects = $a:ty, rels = $r:ty) => {
        pub(super) struct $name;
        impl Entity for $name {
            const NAME: &'static str = stringify!($name);
            type Aspects = $a;
            type Relations = $r;
        }
    };
}
entity_decl!(
    ExternalMapping,
    aspects = (ExternalIdentity,),
    rels = (MappingTarget,)
);
entity_decl!(Principal, aspects = (PrincipalIdentity,), rels = ());
entity_decl!(
    Scope,
    aspects = (ScopeIdentity,),
    rels = (ContainsZero, ContainsFielded)
);
entity_decl!(ZeroNode, aspects = (), rels = (ZeroSelf, ZeroTag));
entity_decl!(
    FieldedNode,
    aspects = (NodeLabel,),
    rels = (FieldedSelf, FieldedTag)
);
entity_decl!(Tag, aspects = (), rels = ());

pub(super) struct ExternalIdentity;
pub(super) struct ExternalIdentityRow {
    ext: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}
impl Record<ExternalIdentityRow> for Bindings {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |r: &ExternalIdentityRow| {
            r.ext.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("MappingStatusField", |r: &ExternalIdentityRow| r.status)
        .equality_queryable(true)
        .build(|ext, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ExternalIdentityRow { ext, status })
        })
    }
}
pub(super) struct PrincipalIdentity;
pub(super) struct PrincipalIdentityRow {
    id: u64,
}
impl Aspect for PrincipalIdentity {
    const NAME: &'static str = "PrincipalIdentity";
    type Row = PrincipalIdentityRow;
}
impl Record<PrincipalIdentityRow> for Bindings {
    fn record() -> impl Shape<PrincipalIdentityRow, Self> {
        field("PrincipalIdentityField", |r: &PrincipalIdentityRow| r.id)
            .equality_queryable(true)
            .read_only()
            .build(|id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalIdentityRow { id })
            })
    }
}
pub(super) struct ScopeIdentity;
pub(super) struct ScopeIdentityRow {
    key: String,
}
impl Aspect for ScopeIdentity {
    const NAME: &'static str = "ScopeIdentity";
    type Row = ScopeIdentityRow;
}
impl Record<ScopeIdentityRow> for Bindings {
    fn record() -> impl Shape<ScopeIdentityRow, Self> {
        field("ScopeKey", |r: &ScopeIdentityRow| r.key.clone())
            .equality_queryable(true)
            .read_only()
            .build(|key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ScopeIdentityRow { key })
            })
    }
}
pub(super) struct NodeLabel;
pub(super) struct NodeLabelRow {
    label: String,
}
impl Aspect for NodeLabel {
    const NAME: &'static str = "NodeLabel";
    type Row = NodeLabelRow;
}
impl Record<NodeLabelRow> for Bindings {
    fn record() -> impl Shape<NodeLabelRow, Self> {
        field("Label", |r: &NodeLabelRow| r.label.clone()).build(
            |label, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(NodeLabelRow { label })
            },
        )
    }
}

pub(super) struct MappingTarget;
pub(super) struct ContainsZero;
pub(super) struct ContainsFielded;
pub(super) struct ZeroSelf;
pub(super) struct ZeroTag;
pub(super) struct FieldedSelf;
pub(super) struct FieldedTag;
macro_rules! rel {
    ($name:ident => $to:ty) => {
        impl Relation for $name {
            const NAME: &'static str = stringify!($name);
            type To = $to;
        }
    };
}
rel!(MappingTarget => Principal);
rel!(ContainsZero => ZeroNode);
rel!(ContainsFielded => FieldedNode);
rel!(ZeroSelf => ZeroNode);
rel!(ZeroTag => Tag);
rel!(FieldedSelf => FieldedNode);
rel!(FieldedTag => Tag);

type ExtField = FieldSelection<
    SelfRefSchema,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
type StatusField = FieldSelection<
    SelfRefSchema,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
type PrincipalIdField = FieldSelection<SelfRefSchema, Principal, PrincipalIdentity, u64>;
pub(super) type ScopeKeyField = FieldSelection<SelfRefSchema, Scope, ScopeIdentity, String>;
pub(super) type LabelField = FieldSelection<SelfRefSchema, FieldedNode, NodeLabel, String>;

pub(super) struct PrincipalBinding;
impl PrincipalBinding {
    fn reference() -> ApplicationPrincipalBindingRef<
        SelfRefSchema,
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
                    ExtField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    StatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(relation(
                    SelfRefSchema,
                    ExternalMapping,
                    MappingTarget,
                )),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

pub(super) struct Params;
pub(super) struct RowValue;
#[derive(Clone)]
pub(super) struct ZeroScopeInput(pub String);
#[derive(Clone)]
pub(super) struct FieldedScopeInput(pub String);
worth_query_structured_value_binding!(pub(super) ParamsBinding for Params { identity: "self-ref.params.v1" });
worth_query_structured_value_binding!(pub(super) RowBinding for RowValue { identity: "self-ref.row.v1" });
worth_query_structured_value_binding!(pub(super) ZeroInputBinding for ZeroScopeInput { identity: "self-ref.zero-input.v1" });
worth_query_structured_value_binding!(pub(super) FieldedInputBinding for FieldedScopeInput { identity: "self-ref.fielded-input.v1" });

pub(super) struct ZeroSelfSlot;
pub(super) struct ZeroTagSlot;
pub(super) struct RootLabelSlot;
pub(super) struct ChildLabelSlot;
pub(super) struct FieldedSelfSlot;
pub(super) struct FieldedTagSlot;
worth_query_portable_type!(ZeroSelfSlot => "self-ref.zero-self.v1");
worth_query_portable_type!(ZeroTagSlot => "self-ref.zero-tag.v1");
worth_query_portable_type!(RootLabelSlot => "self-ref.root-label.v1");
worth_query_portable_type!(ChildLabelSlot => "self-ref.child-label.v1");
worth_query_portable_type!(FieldedSelfSlot => "self-ref.fielded-self.v1");
worth_query_portable_type!(FieldedTagSlot => "self-ref.fielded-tag.v1");

worth_query_application_query!(pub(super) ZeroSourceQuery for SelfRefSchema,
    identity "self-ref.zero-source.v1", parameters ParamsBinding, result RowBinding,
    scope Scope => "Scope", name "zero_source");
worth_query_application_query!(pub(super) FieldedSourceQuery for SelfRefSchema,
    identity "self-ref.fielded-source.v1", parameters ParamsBinding, result RowBinding,
    scope Scope => "Scope", name "fielded_source");

fn zero_scope_key(input: ZeroScopeInput) -> String {
    input.0
}
fn fielded_scope_key(input: FieldedScopeInput) -> String {
    input.0
}

worth_query_query_binding!(pub(super) ZeroSourceBinding for ZeroScopeInput, schema SelfRefSchema,
    identity "self-ref.zero-binding.v1", input ZeroInputBinding,
    query ZeroSourceQuery, parameters ParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result RowBinding, principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ScopeKeyField, field ScopeKeyField::at("ScopeKey"), value zero_scope_key, limits results 64
);
worth_query_query_binding!(pub(super) FieldedSourceBinding for FieldedScopeInput, schema SelfRefSchema,
    identity "self-ref.fielded-binding.v1", input FieldedInputBinding,
    query FieldedSourceQuery, parameters ParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result RowBinding, principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ScopeKeyField, field ScopeKeyField::at("ScopeKey"), value fielded_scope_key, limits results 64
);

worth_query_value_binding!(pub(super) ProbeBinding for String {
    identity: "self-ref.probe.v1",
    scalar: String,
    canonical: ValueDecodeBounds::new(2048, 1024, 8, 8),
});

pub(super) struct ZeroFieldIndex;
impl ApplicationManagedIndexBinding<SelfRefSchema> for ZeroFieldIndex {
    type SourceBinding = ZeroSourceBinding;
    type ProbeBinding = ProbeBinding;
    type Entity = ZeroNode;
    const NAME: &'static str = "zero_field_index";
    const PROVIDER: &'static str = "self-ref.provider.v1";
    const PROVIDER_VERSION: u32 = 1;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
}
pub(super) struct FieldedIndex;
impl ApplicationManagedIndexBinding<SelfRefSchema> for FieldedIndex {
    type SourceBinding = FieldedSourceBinding;
    type ProbeBinding = ProbeBinding;
    type Entity = FieldedNode;
    const NAME: &'static str = "fielded_index";
    const PROVIDER: &'static str = "self-ref.provider.v1";
    const PROVIDER_VERSION: u32 = 1;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
}

mod queries;
use queries::{fielded_source_query, zero_source_query};
