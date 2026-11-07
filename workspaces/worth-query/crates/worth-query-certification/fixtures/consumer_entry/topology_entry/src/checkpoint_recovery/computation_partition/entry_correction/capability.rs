//! An installed grant governs the input that recovery retains.
use super::*;
use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct CorrectionGrantRecords;
impl Fragment for CorrectionGrantRecords {
    const NAME: &'static str = "correction_grants";
    type Bindings = CorrectionGrantBindings;
    type Members = (CorrectionGrant,);
}
pub struct CorrectionGrantBindings;
impl BindingSet for CorrectionGrantBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for CorrectionGrantBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl Scalar<String> for CorrectionGrantBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = false;
}
pub struct CorrectionGrant;
impl Entity for CorrectionGrant {
    const NAME: &'static str = "CorrectionGrant";
    type Aspects = (CorrectionGrantFacts,);
    type Relations = (CorrectionResource, CorrectionParent);
}
impl CorrectionGrant {
    pub fn reference<S: TopologySchemaBinding>() -> ApplicationEntityRef<S, Self> {
        entity(CorrectionGrantRecords, Self)
    }
}
pub struct CorrectionGrantFacts;
impl Aspect for CorrectionGrantFacts {
    const NAME: &'static str = "CorrectionGrantFacts";
    type Row = GrantRow;
}
pub struct GrantRow {
    action: u64,
    purpose: u64,
    status: u64,
    not_before: u64,
    not_after: u64,
    depth: u64,
    scope_key: String,
}
impl fragment::Record<GrantRow> for CorrectionGrantBindings {
    fn record() -> impl Shape<GrantRow, Self> {
        field("CorrectionAction", |row: &GrantRow| row.action)
            .equality_queryable(true)
            .and("CorrectionPurpose", |row: &GrantRow| row.purpose)
            .equality_queryable(true)
            .and("CorrectionStatus", |row: &GrantRow| row.status)
            .equality_queryable(true)
            .and("CorrectionNotBefore", |row: &GrantRow| row.not_before)
            .equality_queryable(true)
            .and("CorrectionNotAfter", |row: &GrantRow| row.not_after)
            .equality_queryable(true)
            .and("CorrectionDepth", |row: &GrantRow| row.depth)
            .equality_queryable(true)
            .and("CorrectionScopeKey", |row: &GrantRow| row.scope_key.clone())
            .equality_queryable(true)
            .build(
                |action,
                 purpose,
                 status,
                 not_before,
                 not_after,
                 depth,
                 scope_key,
                 _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(GrantRow {
                        action,
                        purpose,
                        status,
                        not_before,
                        not_after,
                        depth,
                        scope_key,
                    })
                },
            )
    }
}
pub type CorrectionAction =
    FieldSelection<CorrectionGrantRecords, CorrectionGrant, CorrectionGrantFacts, u64>;
pub type CorrectionPurpose =
    FieldSelection<CorrectionGrantRecords, CorrectionGrant, CorrectionGrantFacts, u64>;
pub type CorrectionStatus =
    FieldSelection<CorrectionGrantRecords, CorrectionGrant, CorrectionGrantFacts, u64>;
pub type CorrectionNotBefore =
    FieldSelection<CorrectionGrantRecords, CorrectionGrant, CorrectionGrantFacts, u64>;
pub type CorrectionNotAfter =
    FieldSelection<CorrectionGrantRecords, CorrectionGrant, CorrectionGrantFacts, u64>;
pub type CorrectionDepth =
    FieldSelection<CorrectionGrantRecords, CorrectionGrant, CorrectionGrantFacts, u64>;
pub type CorrectionScopeKey =
    FieldSelection<CorrectionGrantRecords, CorrectionGrant, CorrectionGrantFacts, String>;
pub struct CorrectionResource;
impl Relation for CorrectionResource {
    const NAME: &'static str = "CorrectionResource";
    type To = Body;
}
impl CorrectionResource {
    pub(in super::super) fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, CorrectionGrant, Body> {
        ApplicationRelationRef::from_schema_identifiers(
            "CorrectionResource",
            "CorrectionGrant",
            "Body",
            relation_integrity::<Self>(),
        )
    }
}
pub struct CorrectionGrantee;
impl Relation for CorrectionGrantee {
    const NAME: &'static str = "CorrectionGrantee";
    type To = CorrectionGrant;
}
impl CorrectionGrantee {
    pub(in super::super) fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Principal, CorrectionGrant> {
        ApplicationRelationRef::from_schema_identifiers(
            "CorrectionGrantee",
            "Principal",
            "CorrectionGrant",
            relation_integrity::<Self>(),
        )
    }
}
pub struct CorrectionGrantor;
impl Relation for CorrectionGrantor {
    const NAME: &'static str = "CorrectionGrantor";
    type To = CorrectionGrant;
}
impl CorrectionGrantor {
    pub(in super::super) fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Principal, CorrectionGrant> {
        ApplicationRelationRef::from_schema_identifiers(
            "CorrectionGrantor",
            "Principal",
            "CorrectionGrant",
            relation_integrity::<Self>(),
        )
    }
}
pub struct CorrectionParent;
impl Relation for CorrectionParent {
    const NAME: &'static str = "CorrectionParent";
    type To = CorrectionGrant;
}
impl CorrectionParent {
    pub(in super::super) fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, CorrectionGrant, CorrectionGrant> {
        ApplicationRelationRef::from_schema_identifiers(
            "CorrectionParent",
            "CorrectionGrant",
            "CorrectionGrant",
            relation_integrity::<Self>(),
        )
    }
}
pub struct CorrectEntryCapability<Schema>(PhantomData<fn() -> Schema>);
impl<Schema> worth_query_decl::facade::portable_identity::WorthQueryPortableType
    for CorrectEntryCapability<Schema>
{
    const PORTABLE_TYPE_NAME: &'static str = "CorrectEntryCapability";
}
impl<Schema: TopologySchemaBinding> ApplicationCapabilityMarkerIdentity
    for CorrectEntryCapability<Schema>
{
    type Schema = Schema;
    const IDENTIFIER: &'static str = "CorrectEntryCapability";
}
impl<Schema: TopologySchemaBinding> CorrectEntryCapability<Schema> {
    pub(in super::super) fn reference() -> ApplicationCapabilityRef<Schema, Self> {
        ApplicationCapabilityRef::from_declaration()
    }
}
pub struct CorrectionContext<Schema>(PhantomData<fn() -> Schema>);
impl<Schema> worth_query_decl::facade::portable_identity::WorthQueryPortableType
    for CorrectionContext<Schema>
{
    const PORTABLE_TYPE_NAME: &'static str = "CorrectionContext";
}
impl<Schema: TopologySchemaBinding> ApplicationCapabilityContextMarkerIdentity
    for CorrectionContext<Schema>
{
    type Schema = Schema;
    const IDENTIFIER: &'static str = "CorrectionContext";
}
impl<Schema: TopologySchemaBinding> CorrectionContext<Schema> {
    pub(in super::super) fn reference() -> ApplicationCapabilityContextRef<Schema, Self> {
        ApplicationCapabilityContextRef::from_declaration()
    }
}
pub struct CorrectionProvenance<Schema>(PhantomData<fn() -> Schema>);
impl<Schema> worth_query_decl::facade::portable_identity::WorthQueryPortableType
    for CorrectionProvenance<Schema>
{
    const PORTABLE_TYPE_NAME: &'static str = "CorrectionProvenance";
}
impl<Schema: TopologySchemaBinding> ApplicationCapabilityProvenanceMarkerIdentity
    for CorrectionProvenance<Schema>
{
    type Schema = Schema;
    const IDENTIFIER: &'static str = "CorrectionProvenance";
}
impl<Schema: TopologySchemaBinding> CorrectionProvenance<Schema> {
    pub(in super::super) fn reference() -> ApplicationCapabilityProvenanceRef<Schema, Self> {
        ApplicationCapabilityProvenanceRef::from_declaration()
    }
}
impl<Schema: TopologySchemaBinding>
    ApplicationCapabilityRequest<Schema, CorrectEntryCapability<Schema>> for EntryCorrection
{
    type Scope = Body;
    type Context = CorrectionContext<Schema>;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Schema, Body, CorrectionContext<Schema>>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let scalar =
            || ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1).unwrap();
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                BodyKey::at("BodyKey"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.scope_key.clone(),
                )
                .unwrap(),
            )
            .unwrap(),
            scalar(),
            scalar(),
            ApplicationCapabilityRequestContext::new(CorrectionContext::<Schema>::reference()),
        ))
    }
}
pub(super) mod contract;
pub(in super::super) use contract::declare;
