//! A real installed capability grant used only by the declared-orbit courts.
use super::super::super::schema::*;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::fragment;
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

pub struct Holder;
impl fragment::Relation for Holder {
    const NAME: &'static str = "Holder";
    type To = Grant;
}
impl Holder {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Principal, Grant> {
        fragment::relation(OrbitRecords::<true>, Principal, Self)
    }
}
pub struct Issuer;
impl fragment::Relation for Issuer {
    const NAME: &'static str = "Issuer";
    type To = Grant;
}
impl Issuer {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Principal, Grant> {
        fragment::relation(OrbitRecords::<true>, Principal, Self)
    }
}
pub struct Resource;
impl fragment::Relation for Resource {
    const NAME: &'static str = "Resource";
    type To = Node;
}
impl Resource {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Grant, Node> {
        fragment::relation(GrantRecords, Grant, Self)
    }
}
pub struct ParentGrant;
impl fragment::Relation for ParentGrant {
    const NAME: &'static str = "ParentGrant";
    type To = Grant;
}
impl ParentGrant {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Grant, Grant> {
        fragment::relation(GrantRecords, Grant, Self)
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct InspectInput {
    pub root: u64,
}
worth_query_portable_type!(InspectInput => "worth.query.orbit.capability-input.v1");
worth_query_structured_value_binding!(pub InspectBinding for InspectInput {identity:"worth.query.orbit.capability-input.v1"});
worth_query_operation!(pub Inspect for S: OrbitModel, input InspectBinding);
worth_query_operation_reads!(Inspect => [Node]);
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
                NodeKey::at("NodeKey"),
                ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(self.root)
                    .unwrap(),
            )
            .unwrap(),
            ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(false).unwrap(),
            ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(false).unwrap(),
            ApplicationCapabilityRequestContext::new(QueryContext::reference()),
        )
        .field(ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(0).unwrap()))
    }
}

pub use super::super::super::schema::records::{
    Grant, GrantAction, GrantActive, GrantDisclosure, GrantFrom, GrantLimit, GrantPurpose,
    GrantUntil, GrantWorkflow,
};
