//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct WalkSchemaRecords;
impl fragment::Fragment for WalkSchemaRecords {
    const NAME: &'static str = "walk_schema_records";
    type Bindings = WalkSchemaRecordsBindings;
    type Members = (ExternalMapping, Principal, Node);
}
pub struct WalkSchemaRecordsBindings;
impl fragment::BindingSet for WalkSchemaRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for WalkSchemaRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for WalkSchemaRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for WalkSchemaRecordsBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<bool> for WalkSchemaRecordsBindings {
    type Binding = BoolApplicationValueBinding;

    const WRITABLE: bool = true;
}
pub struct ExternalMapping;
impl fragment::Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}

pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = (NodeOwner,);
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<WalkSchema, Self> {
        entity(WalkSchemaRecords, Self)
    }
}
pub struct Node;
impl fragment::Entity for Node {
    const NAME: &'static str = "Node";
    type Aspects = (NodeData,);
    type Relations = (GroupRoot, Next);
}
impl Node {
    pub fn reference() -> ApplicationEntityRef<WalkSchema, Self> {
        entity(WalkSchemaRecords, Self)
    }
}
pub struct ExternalIdentity;
impl fragment::Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}

pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for WalkSchemaRecordsBindings {
    fn record() -> impl fragment::Shape<ExternalIdentityValue, Self> {
        fragment::field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |row: &ExternalIdentityValue| {
            row.mapping_status_field
        })
        .equality_queryable(true)
        .build(
            |external_identity_field, mapping_status_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityValue {
                    external_identity_field,
                    mapping_status_field,
                })
            },
        )
    }
}
pub struct PrincipalIdentity;
impl fragment::Aspect for PrincipalIdentity {
    const NAME: &'static str = "PrincipalIdentity";
    type Row = PrincipalIdentityValue;
}

pub struct PrincipalIdentityValue {
    pub principal_identity_field: u64,
}
impl fragment::Record<PrincipalIdentityValue> for WalkSchemaRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalIdentityValue, Self> {
        fragment::field("PrincipalIdentityField", |row: &PrincipalIdentityValue| {
            row.principal_identity_field
        })
        .equality_queryable(true)
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalIdentityValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub struct NodeData;
impl fragment::Aspect for NodeData {
    const NAME: &'static str = "NodeData";
    type Row = NodeDataValue;
}

pub struct NodeDataValue {
    pub node_key: u64,
    pub active: bool,
    pub blocked: bool,
}
impl fragment::Record<NodeDataValue> for WalkSchemaRecordsBindings {
    fn record() -> impl fragment::Shape<NodeDataValue, Self> {
        fragment::field("NodeKey", |row: &NodeDataValue| row.node_key)
            .equality_queryable(true)
            .and("Active", |row: &NodeDataValue| row.active)
            .equality_queryable(true)
            .and("Blocked", |row: &NodeDataValue| row.blocked)
            .equality_queryable(true)
            .build(
                |node_key, active, blocked, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(NodeDataValue {
                        node_key,
                        active,
                        blocked,
                    })
                },
            )
    }
}
pub type ExternalIdentityField = FieldSelection<
    WalkSchemaRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    WalkSchemaRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentityField =
    FieldSelection<WalkSchemaRecords, Principal, PrincipalIdentity, u64>;
pub type NodeKey = FieldSelection<WalkSchemaRecords, Node, NodeData, u64>;
pub type Active = FieldSelection<WalkSchemaRecords, Node, NodeData, bool>;
pub type Blocked = FieldSelection<WalkSchemaRecords, Node, NodeData, bool>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<WalkSchema, Self, ExternalMapping, Principal> {
        relation(WalkSchemaRecords, ExternalMapping, Self)
    }
}
pub struct NodeOwner;
impl fragment::Relation for NodeOwner {
    const NAME: &'static str = "NodeOwner";
    type To = Node;
}
impl NodeOwner {
    pub fn reference() -> ApplicationRelationRef<WalkSchema, Self, Principal, Node> {
        relation(WalkSchemaRecords, Principal, Self)
    }
}
pub struct GroupRoot;
impl fragment::Relation for GroupRoot {
    const NAME: &'static str = "GroupRoot";
    type To = Node;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, None);
}
impl GroupRoot {
    pub fn reference() -> ApplicationRelationRef<WalkSchema, Self, Node, Node> {
        relation(WalkSchemaRecords, Node, Self)
    }
}
pub struct Next;
impl fragment::Relation for Next {
    const NAME: &'static str = "Next";
    type To = Node;
}
impl Next {
    pub fn reference() -> ApplicationRelationRef<WalkSchema, Self, Node, Node> {
        relation(WalkSchemaRecords, Node, Self)
    }
}
