//! Row forms are shared by each concrete relation-law model.
use super::*;
pub struct GraphBindings;
impl BindingSet for GraphBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for GraphBindings {
    type Binding = U64ApplicationValueBinding;
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for GraphBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for GraphBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
pub struct Leaf;
impl Entity for Leaf {
    const NAME: &'static str = "Leaf";
    type Aspects = (LeafFacts,);
    type Relations = ();
}
pub struct Node;
impl Entity for Node {
    const NAME: &'static str = "Node";
    type Aspects = (NodeFacts,);
    type Relations = (Precedes, Feeds);
}
impl Relation for Precedes {
    const NAME: &'static str = "Precedes";
    type To = Node;
}
impl Relation for Feeds {
    const NAME: &'static str = "Feeds";
    type To = Leaf;
}
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = MappingRow;
}
pub struct MappingRow {
    identity: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingRow> for GraphBindings {
    fn record() -> impl Shape<MappingRow, Self> {
        field("ExternalIdentityField", |row: &MappingRow| {
            row.identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |row: &MappingRow| row.status)
        .equality_queryable(true)
        .build(|identity, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(MappingRow { identity, status })
        })
    }
}
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalRow;
}
pub struct PrincipalRow(u64);
impl fragment::Record<PrincipalRow> for GraphBindings {
    fn record() -> impl Shape<PrincipalRow, Self> {
        field("PrincipalKey", |row: &PrincipalRow| row.0)
            .equality_queryable(true)
            .read_only()
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalRow(value))
            })
    }
}
pub struct LeafFacts;
impl Aspect for LeafFacts {
    const NAME: &'static str = "LeafFacts";
    type Row = LeafRow;
}
pub struct LeafRow(u64);
impl fragment::Record<LeafRow> for GraphBindings {
    fn record() -> impl Shape<LeafRow, Self> {
        field("LeafKey", |row: &LeafRow| row.0)
            .equality_queryable(true)
            .read_only()
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(LeafRow(value))
            })
    }
}
pub struct NodeFacts;
impl Aspect for NodeFacts {
    const NAME: &'static str = "NodeFacts";
    type Row = NodeRow;
}
pub struct NodeRow(u64);
impl fragment::Record<NodeRow> for GraphBindings {
    fn record() -> impl Shape<NodeRow, Self> {
        field("NodeKey", |row: &NodeRow| row.0)
            .equality_queryable(true)
            .read_only()
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(NodeRow(value))
            })
    }
}
pub type ExternalIdentityField = FieldSelection<
    UnrestrictedRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    UnrestrictedRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalKey = FieldSelection<UnrestrictedRecords, Principal, PrincipalFacts, u64>;
pub type NodeKey = FieldSelection<UnrestrictedRecords, Node, NodeFacts, u64>;
pub type LeafKey = FieldSelection<UnrestrictedRecords, Leaf, LeafFacts, u64>;
impl ExternalMapping {
    pub const fn reference<S: GraphSchemaBinding>() -> ApplicationEntityRef<S, Self> {
        ApplicationEntityRef::from_schema_identifier(<Self as Entity>::NAME)
    }
}
impl Principal {
    pub const fn reference<S: GraphSchemaBinding>() -> ApplicationEntityRef<S, Self> {
        ApplicationEntityRef::from_schema_identifier(<Self as Entity>::NAME)
    }
}
impl Leaf {
    pub const fn reference<S: GraphSchemaBinding>() -> ApplicationEntityRef<S, Self> {
        ApplicationEntityRef::from_schema_identifier(<Self as Entity>::NAME)
    }
}
