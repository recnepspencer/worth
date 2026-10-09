//! An installed grant governs the input that recovery retains.
use super::*;
use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::{worth_query_aspect, worth_query_entity, worth_query_field};
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap;
worth_query_entity!(pub(in super::super) CorrectionGrant for Schema: TopologySchemaBinding);
worth_query_aspect!(pub(in super::super) CorrectionGrantFacts for Schema: TopologySchemaBinding, CorrectionGrant; identity = AspectIdentity(0x9174_10a1), revision = AspectContractRevision(1),);
macro_rules! grant_fields { ($($name:ident),+) => {$(worth_query_field!(pub(in super::super) $name for Schema: TopologySchemaBinding, CorrectionGrant, CorrectionGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);)+}; }
grant_fields!(
    CorrectionAction,
    CorrectionPurpose,
    CorrectionStatus,
    CorrectionNotBefore,
    CorrectionNotAfter,
    CorrectionDepth
);
worth_query_field!(pub(in super::super) CorrectionScopeKey for Schema: TopologySchemaBinding, CorrectionGrant, CorrectionGrantFacts: String => StringApplicationValueBinding, read_only, equality);
pub(in super::super) struct CorrectionResource;
impl CorrectionResource {
    pub(in super::super) fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, CorrectionGrant, Body> {
        ApplicationRelationRef::from_schema_identifiers(
            "CorrectionResource",
            "CorrectionGrant",
            "Body",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub(in super::super) struct CorrectionGrantee;
impl CorrectionGrantee {
    pub(in super::super) fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Principal, CorrectionGrant> {
        ApplicationRelationRef::from_schema_identifiers(
            "CorrectionGrantee",
            "Principal",
            "CorrectionGrant",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub(in super::super) struct CorrectionGrantor;
impl CorrectionGrantor {
    pub(in super::super) fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Principal, CorrectionGrant> {
        ApplicationRelationRef::from_schema_identifiers(
            "CorrectionGrantor",
            "Principal",
            "CorrectionGrant",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub(in super::super) struct CorrectionParent;
impl CorrectionParent {
    pub(in super::super) fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, CorrectionGrant, CorrectionGrant> {
        ApplicationRelationRef::from_schema_identifiers(
            "CorrectionParent",
            "CorrectionGrant",
            "CorrectionGrant",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub(in super::super) struct CorrectEntryCapability<Schema>(PhantomData<fn() -> Schema>);
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
pub(in super::super) struct CorrectionContext<Schema>(PhantomData<fn() -> Schema>);
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
pub(in super::super) struct CorrectionProvenance<Schema>(PhantomData<fn() -> Schema>);
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
                BodyKey::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.scope_key.clone(),
                )
                .unwrap(),
            ),
            scalar(),
            scalar(),
            ApplicationCapabilityRequestContext::new(CorrectionContext::<Schema>::reference()),
        ))
    }
}
mod contract;
pub(in super::super) use contract::declare;
pub(in super::super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>,
    scope: &str,
) {
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationEntityKey as Key, WorthQueryApplicationEntitySeed as Seed,
        WorthQueryApplicationRelationSeed as Relation,
    };
    fn key<Entity>(s: &str) -> Key<CheckpointSchema, Entity> {
        Key::new(s).unwrap()
    }
    graph
        .bind_entity(
            Seed::new(CorrectionGrant::reference(), key("entry-correction-grant"))
                .field(CorrectionAction::reference(), 1)
                .field(CorrectionPurpose::reference(), 1)
                .field(CorrectionStatus::reference(), 1)
                .field(CorrectionScopeKey::reference(), scope.to_owned())
                .field(CorrectionNotBefore::reference(), 0)
                .field(CorrectionNotAfter::reference(), u64::MAX)
                .field(CorrectionDepth::reference(), 0),
        )
        .unwrap();
    graph
        .bind_relation(Relation::new(
            CorrectionResource::reference(),
            "correction-resource",
            key("entry-correction-grant"),
            key(scope),
        ))
        .unwrap();
    graph
        .bind_relation(Relation::new(
            CorrectionGrantee::reference(),
            "correction-grantee",
            key("model-owner"),
            key("entry-correction-grant"),
        ))
        .unwrap();
    graph
        .bind_relation(Relation::new(
            CorrectionGrantor::reference(),
            "correction-grantor",
            key("model-owner"),
            key("entry-correction-grant"),
        ))
        .unwrap();
}
