//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
use worth_query_declaration::facade::authentication::*;
pub struct ComponentRecords;
impl fragment::Fragment for ComponentRecords {
    const NAME: &'static str = "component_records";
    type Bindings = ComponentRecordsBindings;
    type Members = (Node, Mapping, Principal);
}
pub struct ComponentRecordsBindings;
impl fragment::BindingSet for ComponentRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for ComponentRecordsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for ComponentRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for ComponentRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
pub struct Node;
impl fragment::Entity for Node {
    const NAME: &'static str = "Node";
    type Aspects = (NodeData,);
    type Relations = (A, B);
}
impl Node {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ComponentRecords, Self)
    }
}
pub struct Mapping;
impl fragment::Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (MappingData,);
    type Relations = (super::principal_binding::MappingTarget,);
}

pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalData,);
    type Relations = ();
}

pub struct NodeData;
impl fragment::Aspect for NodeData {
    const NAME: &'static str = "NodeData";
    type Row = NodeDataValue;
}

pub struct NodeDataValue {
    pub node_key: u64,
}
impl fragment::Record<NodeDataValue> for ComponentRecordsBindings {
    fn record() -> impl fragment::Shape<NodeDataValue, Self> {
        fragment::field("NodeKey", |row: &NodeDataValue| row.node_key).build(
            |node_key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(NodeDataValue { node_key })
            },
        )
    }
}
pub struct MappingData;
impl fragment::Aspect for MappingData {
    const NAME: &'static str = "MappingData";
    type Row = MappingDataValue;
}

pub struct MappingDataValue {
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingDataValue> for ComponentRecordsBindings {
    fn record() -> impl fragment::Shape<MappingDataValue, Self> {
        fragment::field("ExternalIdentity", |row: &MappingDataValue| {
            row.external_identity.clone()
        })
        .and("MappingStatus", |row: &MappingDataValue| row.mapping_status)
        .build(
            |external_identity, mapping_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingDataValue {
                    external_identity,
                    mapping_status,
                })
            },
        )
    }
}
pub struct PrincipalData;
impl fragment::Aspect for PrincipalData {
    const NAME: &'static str = "PrincipalData";
    type Row = PrincipalDataValue;
}

pub struct PrincipalDataValue {
    pub principal_key: u64,
}
impl fragment::Record<PrincipalDataValue> for ComponentRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalDataValue, Self> {
        fragment::field("PrincipalKey", |row: &PrincipalDataValue| row.principal_key).build(
            |principal_key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalDataValue { principal_key })
            },
        )
    }
}
pub type NodeKey = FieldSelection<ComponentRecords, Node, NodeData, u64>;
pub type ExternalIdentity =
    FieldSelection<ComponentRecords, Mapping, MappingData, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatus =
    FieldSelection<ComponentRecords, Mapping, MappingData, WorthQueryPrincipalMappingStatus>;
pub type PrincipalKey = FieldSelection<ComponentRecords, Principal, PrincipalData, u64>;
