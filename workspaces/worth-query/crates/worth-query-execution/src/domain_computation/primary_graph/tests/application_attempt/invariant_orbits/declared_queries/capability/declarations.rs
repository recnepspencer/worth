//! A real installed capability grant used only by the declared-orbit courts.
use super::super::super::schema::*;
use std::marker::PhantomData;
use worth_query_declaration::{
    facade::{application_capability::*, application_schema::*, portable_identity::*},
    *,
};
pub struct ReadOrbit<S>(PhantomData<S>);
impl<S> WorthQueryPortableType for ReadOrbit<S> {
    const PORTABLE_TYPE_NAME: &'static str = "worth.query.orbit.ReadOrbit.v1";
}
impl<S: OrbitModel> ApplicationCapabilityMarkerIdentity for ReadOrbit<S> {
    type Schema = S;
    const IDENTIFIER: &'static str = "ReadOrbit";
}
impl<S: OrbitModel> ReadOrbit<S> {
    pub fn reference() -> ApplicationCapabilityRef<S, Self> {
        ApplicationCapabilityRef::from_declaration()
    }
}
pub struct QueryContext<S>(PhantomData<S>);
impl<S> WorthQueryPortableType for QueryContext<S> {
    const PORTABLE_TYPE_NAME: &'static str = "worth.query.orbit.QueryContext.v1";
}
impl<S: OrbitModel> ApplicationCapabilityContextMarkerIdentity for QueryContext<S> {
    type Schema = S;
    const IDENTIFIER: &'static str = "QueryContext";
}
impl<S: OrbitModel> QueryContext<S> {
    pub fn reference() -> ApplicationCapabilityContextRef<S, Self> {
        ApplicationCapabilityContextRef::from_declaration()
    }
}
pub struct QueryProvenance<S>(PhantomData<S>);
impl<S> WorthQueryPortableType for QueryProvenance<S> {
    const PORTABLE_TYPE_NAME: &'static str = "worth.query.orbit.QueryProvenance.v1";
}
impl<S: OrbitModel> ApplicationCapabilityProvenanceMarkerIdentity for QueryProvenance<S> {
    type Schema = S;
    const IDENTIFIER: &'static str = "QueryProvenance";
}
impl<S: OrbitModel> QueryProvenance<S> {
    pub fn reference() -> ApplicationCapabilityProvenanceRef<S, Self> {
        ApplicationCapabilityProvenanceRef::from_declaration()
    }
}
worth_query_entity!(pub Grant for S: OrbitModel);
worth_query_aspect!(pub GrantFacts for S: OrbitModel, Grant; identity = AspectIdentity(0x9181_4110), revision = AspectContractRevision(1));
worth_query_field!(pub GrantAction for S: OrbitModel, Grant, GrantFacts: bool => BoolApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantPurpose for S: OrbitModel, Grant, GrantFacts: bool => BoolApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantActive for S: OrbitModel, Grant, GrantFacts: bool => BoolApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantWorkflow for S: OrbitModel, Grant, GrantFacts: bool => BoolApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantDisclosure for S: OrbitModel, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantFrom for S: OrbitModel, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantUntil for S: OrbitModel, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantLimit for S: OrbitModel, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, no_equality);
pub struct Holder;
impl Holder {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Principal, Grant> {
        ApplicationRelationRef::from_schema_identifiers(
            "Holder",
            "Principal",
            "Grant",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub struct Issuer;
impl Issuer {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Principal, Grant> {
        ApplicationRelationRef::from_schema_identifiers(
            "Issuer",
            "Principal",
            "Grant",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub struct Resource;
impl Resource {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Grant, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            "Resource",
            "Grant",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub struct ParentGrant;
impl ParentGrant {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Grant, Grant> {
        ApplicationRelationRef::from_schema_identifiers(
            "ParentGrant",
            "Grant",
            "Grant",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct InspectInput {
    pub root: u64,
}
worth_query_portable_type!(InspectInput => "worth.query.orbit.capability-input.v1");
worth_query_structured_value_binding!(pub InspectBinding for InspectInput {identity:"worth.query.orbit.capability-input.v1"});
worth_query_operation!(pub Inspect for S: OrbitModel, input InspectBinding);
worth_query_operation_reads!(Inspect => [Node, NodeKey]);
impl<S: OrbitModel> ApplicationCapabilityRequest<S, ReadOrbit<S>> for InspectInput {
    type Scope = Node;
    type Context = QueryContext<S>;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<S, Node, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                NodeKey::reference(),
                ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(self.root)
                    .unwrap(),
            ),
            ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(false).unwrap(),
            ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(false).unwrap(),
            ApplicationCapabilityRequestContext::new(QueryContext::reference()),
        )
        .field(ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(0).unwrap()))
    }
}
