//! Schema and source query for Directory Home optional-aspect absence reopen courts.
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
use worth_query_host::facade::{
    declaration::{application_query::*, authentication::*},
    primary_graph::*,
    *,
};

#[derive(Clone, Copy)]
pub struct OptionalAspectSchema;
impl Fragment for OptionalAspectSchema {
    const NAME: &'static str = "OptionalAspectReopenSchema";
    type Bindings = NoBindings;
    type Members = (OptionalAspectContribution,);
}
impl ApplicationSchema for OptionalAspectSchema {
    const OWNER: &'static str = "worth.query.test.optional-aspect-reopen";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}
impl ApplicationSchemaComposition for OptionalAspectSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct OptionalAspectContribution;
impl ApplicationSchemaContribution<OptionalAspectSchema> for OptionalAspectContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("optional-aspect-reopen.contribution.v1");
    type Members = (Records, super::program::Contracts);
}

#[derive(Clone, Copy)]
pub struct Records;
impl Fragment for Records {
    const NAME: &'static str = "optional_aspect_records";
    type Bindings = RecordBindings;
    type Members = (ExternalMapping, Principal, Anchor, Derived);
}
pub struct RecordBindings;
impl BindingSet for RecordBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for RecordBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for RecordBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for RecordBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<String> for RecordBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = true;
}

#[derive(Clone, Copy)]
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<OptionalAspectSchema, Self> {
        entity(Records, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<OptionalAspectSchema, Self> {
        entity(Records, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Anchor;
impl Entity for Anchor {
    const NAME: &'static str = "Anchor";
    type Aspects = (AnchorFacts, Option<OptionalSource>);
    type Relations = ();
}
impl Anchor {
    pub fn reference() -> ApplicationEntityRef<OptionalAspectSchema, Self> {
        entity(Records, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Derived;
impl Entity for Derived {
    const NAME: &'static str = "Derived";
    type Aspects = (DerivedFacts,);
    type Relations = ();
}
impl Derived {
    pub fn reference() -> ApplicationEntityRef<OptionalAspectSchema, Self> {
        entity(Records, Self)
    }
}

pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}
pub struct ExternalIdentityRow {
    pub external_id: WorthQueryExternalPrincipalIdentity,
    pub status: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityRow> for RecordBindings {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |r: &ExternalIdentityRow| {
            r.external_id.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |r: &ExternalIdentityRow| r.status)
        .build(|external_id, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ExternalIdentityRow {
                external_id,
                status,
            })
        })
    }
}
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<OptionalAspectSchema, Self, ExternalMapping, Principal> {
        relation(Records, ExternalMapping, Self)
    }
}
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalRow;
}
pub struct PrincipalRow {
    pub id: u64,
}
impl Record<PrincipalRow> for RecordBindings {
    fn record() -> impl Shape<PrincipalRow, Self> {
        field("PrincipalIdentityField", |r: &PrincipalRow| r.id)
            .equality_queryable(true)
            .build(|id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalRow { id })
            })
    }
}
pub struct AnchorFacts;
impl Aspect for AnchorFacts {
    const NAME: &'static str = "AnchorFacts";
    type Row = AnchorRow;
}
pub struct AnchorRow {
    pub key: String,
}
impl Record<AnchorRow> for RecordBindings {
    fn record() -> impl Shape<AnchorRow, Self> {
        field("AnchorKey", |r: &AnchorRow| r.key.clone())
            .equality_queryable(true)
            .build(|key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AnchorRow { key })
            })
    }
}
pub struct OptionalSource;
impl Aspect for OptionalSource {
    const NAME: &'static str = "OptionalSource";
    type Row = OptionalSourceRow;
}
pub struct OptionalSourceRow {
    pub content: String,
    pub selection: Option<String>,
}
impl Record<OptionalSourceRow> for RecordBindings {
    fn record() -> impl Shape<OptionalSourceRow, Self> {
        field("SourceContent", |r: &OptionalSourceRow| r.content.clone())
            .and("SourceSelection", |r: &OptionalSourceRow| {
                r.selection.clone()
            })
            .build(|content, selection, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(OptionalSourceRow { content, selection })
            })
    }
}
pub struct DerivedFacts;
impl Aspect for DerivedFacts {
    const NAME: &'static str = "DerivedFacts";
    type Row = DerivedRow;
}
pub struct DerivedRow {
    pub key: String,
    pub tag: String,
}
impl Record<DerivedRow> for RecordBindings {
    fn record() -> impl Shape<DerivedRow, Self> {
        field("DerivedKey", |r: &DerivedRow| r.key.clone())
            .equality_queryable(true)
            .and("DerivedTag", |r: &DerivedRow| r.tag.clone())
            .build(|key, tag, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(DerivedRow { key, tag })
            })
    }
}

pub type ExternalIdentityField =
    FieldSelection<Records, ExternalMapping, ExternalIdentity, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatusField =
    FieldSelection<Records, ExternalMapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentityField = FieldSelection<Records, Principal, PrincipalFacts, u64>;
pub type AnchorKey = FieldSelection<Records, Anchor, AnchorFacts, String>;
pub type SourceContent = FieldSelection<Records, Anchor, OptionalSource, String>;
pub type SourceSelection = FieldSelection<Records, Anchor, OptionalSource, String>;
pub type DerivedKey = FieldSelection<Records, Derived, DerivedFacts, String>;
pub type DerivedTag = FieldSelection<Records, Derived, DerivedFacts, String>;

pub struct OperatorPrincipalBinding;
impl OperatorPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        OptionalAspectSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "OperatorPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyRow(pub String);
pub struct QueryParams;
pub struct KeySlot;
worth_query_portable_type!(KeySlot => "optional-aspect-reopen.key-slot.v1");
worth_query_structured_value_binding!(pub QueryParamsBinding for QueryParams { identity: "optional-aspect-reopen.query-params.v1" });
worth_query_structured_value_binding!(pub KeyRowBinding for KeyRow { identity: "optional-aspect-reopen.key-row.v1" });
worth_query_application_query!(pub AnchorQuery for OptionalAspectSchema, identity "optional-aspect-reopen.anchor-query.v1",
    parameters QueryParamsBinding, result KeyRowBinding, scope Anchor => "Anchor", name "anchor_key");
pub fn key_field(
) -> ApplicationQueryResultFieldRef<AnchorQuery, KeySlot, OptionalAspectSchema, AnchorKey> {
    ApplicationQueryResultFieldRef::new("key", AnchorKey::at("AnchorKey"))
}
pub fn anchor_query_definition(
) -> ApplicationQueryDefinitionBuilder<OptionalAspectSchema, AnchorQuery, QueryParams, KeyRow, Anchor>
{
    let shape = ApplicationQueryResultShapeBuilder::<
        OptionalAspectSchema,
        AnchorQuery,
        Anchor,
        KeyRow,
        KeyRowBinding,
    >::new(Anchor::reference())
    .field(key_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(AnchorQuery::reference())
        .root(Anchor::reference())
        .scope(Anchor::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
impl WorthQueryApplicationProjection<OptionalAspectSchema, AnchorQuery> for KeyRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, OptionalAspectSchema, AnchorQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        row.field(key_field()).map(KeyRow)
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadAnchor {
    pub key: String,
}
worth_query_structured_value_binding!(pub ReadAnchorBinding for ReadAnchor { identity: "optional-aspect-reopen.read-anchor.v1" });
pub struct AnchorReadBinding;
pub type AnchorScope = ApplicationQueryFieldScope<AnchorKey>;
impl ApplicationQueryBinding<OptionalAspectSchema> for AnchorReadBinding {
    type Input = ReadAnchor;
    type InputBinding = ReadAnchorBinding;
    type Query = AnchorQuery;
    type ParameterBinding = QueryParamsBinding;
    type ResultBinding = KeyRowBinding;
    type ScopeBinding = AnchorScope;
    type PrincipalBinding = OperatorPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "optional-aspect-reopen.anchor-read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> AnchorKey {
        AnchorKey::at("AnchorKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        OptionalAspectSchema,
        OperatorPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        OperatorPrincipalBinding::reference()
    }
}
impl ApplicationQueryIntent<OptionalAspectSchema> for ReadAnchor {
    type Binding = AnchorReadBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<AnchorQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> AnchorScope {
        AnchorScope::new(AnchorKey::at("AnchorKey"), self.key)
    }
}
