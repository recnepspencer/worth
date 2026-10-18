//! An installed grant governs the input that recovery retains.
use super::*;
use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::{worth_query_aspect, worth_query_entity, worth_query_field};
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
pub(super) mod contract;
pub(in super::super) use contract::declare;
