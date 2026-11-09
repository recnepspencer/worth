//! Authored members are separate records with one owner and optional chain links.
use super::*;
use worth_query_decl::facade::application_schema::fragment::*;
pub struct Member;
impl Entity for Member {
    const NAME: &'static str = "AuthoredMember";
    type Aspects = (MemberFacts,);
    type Relations = (OwnedBy, Next, Operand);
}
impl Member {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(SourceModel, Self)
    }
}
pub struct MemberFacts;
impl Aspect for MemberFacts {
    const NAME: &'static str = "MemberFacts";
    type Row = MemberValue;
}
pub struct MemberValue {
    pub key: String,
    pub value: u64,
}
impl Record<MemberValue> for SourceModelBindings {
    fn record() -> impl Shape<MemberValue, Self> {
        field("MemberKey", |row: &MemberValue| row.key.clone())
            .equality_queryable(true)
            .and("MemberValue", |row: &MemberValue| row.value)
            .build(|key, value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MemberValue { key, value })
            })
    }
}
pub type MemberKey = FieldSelection<SourceModel, Member, MemberFacts, String>;
pub type Value = FieldSelection<SourceModel, Member, MemberFacts, u64>;
pub struct OwnedBy;
impl Relation for OwnedBy {
    const NAME: &'static str = "OwnedBy";
    type To = Source;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, None);
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
}
impl OwnedBy {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Member, Source> {
        relation(SourceModel, Member, Self)
    }
}
pub struct Next;
impl Relation for Next {
    const NAME: &'static str = "MemberNext";
    type To = Member;
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
}
impl Next {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Member, Member> {
        relation(SourceModel, Member, Self)
    }
}
pub struct AnchorFamily;
impl ApplicationAuthoredCorrectionFamily<Schema> for AnchorFamily {
    const IDENTITY: &'static str = "source-subgraph.anchor.v1";
    type Entity = Source;
    fn logical_key() -> ApplicationAuthoredLogicalKey<Schema, Source> {
        ApplicationAuthoredLogicalKey::from_field(SourceKey::at("SourceKey"))
    }
}
pub struct MemberFamily;
impl ApplicationAuthoredCorrectionFamily<Schema> for MemberFamily {
    const IDENTITY: &'static str = "source-subgraph.member.v1";
    type Entity = Member;
    fn logical_key() -> ApplicationAuthoredLogicalKey<Schema, Member> {
        ApplicationAuthoredLogicalKey::from_field(MemberKey::at("MemberKey"))
    }
}

pub struct Head;
impl Relation for Head {
    const NAME: &'static str = "RosterHead";
    type To = Member;
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
}
impl Head {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Source, Member> {
        relation(SourceModel, Source, Self)
    }
}

pub struct Operand;
impl Relation for Operand {
    const NAME: &'static str = "MemberOperand";
    type To = Member;
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
}
impl Operand {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Member, Member> {
        relation(SourceModel, Member, Self)
    }
}
