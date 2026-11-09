//! Native facts and relations for the relation-only inverse court.
use super::*;
pub struct StorageModel;
impl Fragment for StorageModel {
    const NAME: &'static str = "relation_model";
    type Bindings = Bindings;
    type Members = (Mapping, Principal, Node);
}
pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for Bindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Bindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
pub struct Mapping;
impl Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (MappingFacts,);
    type Relations = (MappingTarget,);
}
impl Mapping {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(StorageModel, Self)
    }
}
pub struct MappingFacts;
impl Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingFactsRow;
}
pub struct MappingFactsRow(
    WorthQueryExternalPrincipalIdentity,
    WorthQueryPrincipalMappingStatus,
);
impl fragment::Record<MappingFactsRow> for Bindings {
    fn record() -> impl Shape<MappingFactsRow, Self> {
        fragment::field("ExternalId", |row: &MappingFactsRow| row.0.clone())
            .equality_queryable(true)
            .read_only()
            .and("Status", |row: &MappingFactsRow| row.1)
            .equality_queryable(true)
            .build(|v0, v1, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingFactsRow(v0, v1))
            })
    }
}
pub type ExternalId =
    FieldSelection<StorageModel, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status =
    FieldSelection<StorageModel, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(StorageModel, Self)
    }
}
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsRow;
}
pub struct PrincipalFactsRow(u64);
impl fragment::Record<PrincipalFactsRow> for Bindings {
    fn record() -> impl Shape<PrincipalFactsRow, Self> {
        fragment::field("PrincipalId", |row: &PrincipalFactsRow| row.0)
            .equality_queryable(true)
            .read_only()
            .build(|v0, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsRow(v0))
            })
    }
}
pub type PrincipalId = FieldSelection<StorageModel, Principal, PrincipalFacts, u64>;
pub struct Node;
impl Entity for Node {
    const NAME: &'static str = "Node";
    type Aspects = (Facts,);
    type Relations = (Unrelated,);
}
impl Node {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(StorageModel, Self)
    }
}
pub struct Facts;
impl Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = FactsRow;
}
pub struct FactsRow(u64);
impl fragment::Record<FactsRow> for Bindings {
    fn record() -> impl Shape<FactsRow, Self> {
        fragment::field("Id", |row: &FactsRow| row.0)
            .equality_queryable(true)
            .read_only()
            .unique()
            .build(|v0, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(FactsRow(v0))
            })
    }
}
pub type Id = FieldSelection<StorageModel, Node, Facts, u64>;
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Mapping, Principal> {
        relation(StorageModel, Mapping, Self)
    }
}
pub struct Holds;
impl Relation for Holds {
    const NAME: &'static str = "Holds";
    type To = Node;
}
impl Holds {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Node, Node> {
        relation(LinksFeature, Node, Self)
    }
}
pub struct Unrelated;
impl Relation for Unrelated {
    const NAME: &'static str = "Unrelated";
    type To = Node;
}
impl Unrelated {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Node, Node> {
        relation(StorageModel, Node, Self)
    }
}

pub struct LinksFeature;
impl Fragment for LinksFeature {
    const NAME: &'static str = "links_feature";
    type Bindings = Bindings;
    type Members = (Extend<Node, (), (Holds,)>,);
}
