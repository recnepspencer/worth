//! Authored orbit and ordinary-edge profiles reuse the same checked row bindings.
use super::super::{
    component_invariants::declaration::SpareCoupling,
    declared_queries::capability::declarations::{Holder, Issuer, ParentGrant, Resource},
};
use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
pub struct OrbitRecords<const ORBIT: bool = true>;
impl<const O: bool> Fragment for OrbitRecords<O> {
    const NAME: &'static str = "orbit_model";
    type Bindings = OrbitBindings;
    type Members = (ExternalMapping, Principal, Node<O>, Spare);
}
pub struct GrantRecords;
impl Fragment for GrantRecords {
    const NAME: &'static str = "orbit_grants";
    type Bindings = GrantBindings;
    type Members = (Grant,);
}
pub struct OrbitBindings;
impl BindingSet for OrbitBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for OrbitBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for OrbitBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl Scalar<u64> for OrbitBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl Scalar<bool> for OrbitBindings {
    type Binding = BoolApplicationValueBinding;

    const WRITABLE: bool = true;
}
pub struct GrantBindings;
impl BindingSet for GrantBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<bool> for GrantBindings {
    type Binding = BoolApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl Scalar<u64> for GrantBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
#[derive(Clone, Copy, Debug)]
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}

pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}

pub struct ExternalIdentityRow {
    pub value_0: WorthQueryExternalPrincipalIdentity,
    pub value_1: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityRow> for OrbitBindings {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityRow| {
            row.value_0.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |row: &ExternalIdentityRow| {
            row.value_1
        })
        .equality_queryable(true)
        .build(|value_0, value_1, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ExternalIdentityRow { value_0, value_1 })
        })
    }
}
pub type ExternalIdentityField = FieldSelection<
    OrbitRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    OrbitRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
#[derive(Clone, Copy, Debug)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = (Holder, Issuer);
}
impl Principal {
    pub fn reference<S: OrbitModel>() -> ApplicationEntityRef<S, Self> {
        entity(OrbitRecords::<true>, Self)
    }
}
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalRow;
}

pub struct PrincipalRow {
    pub value_0: u64,
}
impl fragment::Record<PrincipalRow> for OrbitBindings {
    fn record() -> impl Shape<PrincipalRow, Self> {
        field("PrincipalKey", |row: &PrincipalRow| row.value_0)
            .equality_queryable(true)
            .build(|value_0, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalRow { value_0 })
            })
    }
}
pub type PrincipalKey = FieldSelection<OrbitRecords, Principal, PrincipalFacts, u64>;
#[derive(Clone, Copy, Debug)]
pub struct Node<const O: bool = true>;
impl<const O: bool> Entity for Node<O> {
    const NAME: &'static str = "Node";
    type Aspects = (NodeData,);
    type Relations = (
        Next<O>,
        super::super::decision::components::binding::Visible,
    );
}
impl Node {
    pub fn reference<S: OrbitModel>() -> ApplicationEntityRef<S, Self> {
        entity(OrbitRecords::<true>, Self)
    }
}
pub struct NodeData;
impl Aspect for NodeData {
    const NAME: &'static str = "NodeData";
    type Row = NodeDataValue;
}

pub struct NodeDataValue {
    pub value_0: u64,
    pub value_1: bool,
}
impl fragment::Record<NodeDataValue> for OrbitBindings {
    fn record() -> impl Shape<NodeDataValue, Self> {
        field("NodeKey", |row: &NodeDataValue| row.value_0)
            .equality_queryable(true)
            .and("Marked", |row: &NodeDataValue| row.value_1)
            .equality_queryable(true)
            .build(|value_0, value_1, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(NodeDataValue { value_0, value_1 })
            })
    }
}
pub type NodeKey = FieldSelection<OrbitRecords, Node, NodeData, u64>;
pub type Marked = FieldSelection<OrbitRecords, Node, NodeData, bool>;
#[derive(Clone, Copy, Debug)]
pub struct Spare;
impl Entity for Spare {
    const NAME: &'static str = "Spare";
    type Aspects = (SpareData,);
    type Relations = (SpareCoupling,);
}
impl Spare {
    pub fn reference<S: OrbitModel>() -> ApplicationEntityRef<S, Self> {
        entity(OrbitRecords::<true>, Self)
    }
}
pub struct SpareData;
impl Aspect for SpareData {
    const NAME: &'static str = "SpareData";
    type Row = SpareRow;
}

pub struct SpareRow {
    pub value_0: u64,
}
impl fragment::Record<SpareRow> for OrbitBindings {
    fn record() -> impl Shape<SpareRow, Self> {
        field("SpareKey", |row: &SpareRow| row.value_0)
            .equality_queryable(true)
            .build(|value_0, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SpareRow { value_0 })
            })
    }
}
pub type SpareKey = FieldSelection<OrbitRecords, Spare, SpareData, u64>;
#[derive(Clone, Copy, Debug)]
pub struct Grant;
impl Entity for Grant {
    const NAME: &'static str = "Grant";
    type Aspects = (GrantFacts,);
    type Relations = (Resource, ParentGrant);
}
impl Grant {
    pub fn reference<S: OrbitModel>() -> ApplicationEntityRef<S, Self> {
        entity(GrantRecords, Self)
    }
}
pub struct GrantFacts;
impl Aspect for GrantFacts {
    const NAME: &'static str = "GrantFacts";
    type Row = GrantRow;
}

pub struct GrantRow {
    pub value_0: bool,
    pub value_1: bool,
    pub value_2: bool,
    pub value_3: bool,
    pub value_4: u64,
    pub value_5: u64,
    pub value_6: u64,
    pub value_7: u64,
}
impl fragment::Record<GrantRow> for GrantBindings {
    fn record() -> impl Shape<GrantRow, Self> {
        field("GrantAction", |row: &GrantRow| row.value_0)
            .and("GrantPurpose", |row: &GrantRow| row.value_1)
            .and("GrantActive", |row: &GrantRow| row.value_2)
            .and("GrantWorkflow", |row: &GrantRow| row.value_3)
            .and("GrantDisclosure", |row: &GrantRow| row.value_4)
            .and("GrantFrom", |row: &GrantRow| row.value_5)
            .and("GrantUntil", |row: &GrantRow| row.value_6)
            .and("GrantLimit", |row: &GrantRow| row.value_7)
            .build(
                |value_0,
                 value_1,
                 value_2,
                 value_3,
                 value_4,
                 value_5,
                 value_6,
                 value_7,
                 _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(GrantRow {
                        value_0,
                        value_1,
                        value_2,
                        value_3,
                        value_4,
                        value_5,
                        value_6,
                        value_7,
                    })
                },
            )
    }
}
pub type GrantAction = FieldSelection<GrantRecords, Grant, GrantFacts, bool>;
pub type GrantPurpose = FieldSelection<GrantRecords, Grant, GrantFacts, bool>;
pub type GrantActive = FieldSelection<GrantRecords, Grant, GrantFacts, bool>;
pub type GrantWorkflow = FieldSelection<GrantRecords, Grant, GrantFacts, bool>;
pub type GrantDisclosure = FieldSelection<GrantRecords, Grant, GrantFacts, u64>;
pub type GrantFrom = FieldSelection<GrantRecords, Grant, GrantFacts, u64>;
pub type GrantUntil = FieldSelection<GrantRecords, Grant, GrantFacts, u64>;
pub type GrantLimit = FieldSelection<GrantRecords, Grant, GrantFacts, u64>;
