//! Native fragment membership and original record shapes for this fixture.
use super::*;
#[derive(Clone, Copy)]
pub struct Storage;
impl Fragment for Storage {
    const NAME: &'static str = "storage";
    type Bindings = Bindings;
    type Members = (
        Mapping,
        Principal,
        Model,
        Member,
        FirstOutput,
        SecondOutput,
        ChildOutput,
        Cap,
    );
}
mod bindings;
pub use bindings::Bindings;
#[derive(Clone, Copy)]
pub struct Mapping;
impl Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (MappingFacts,);
    type Relations = (MappingTarget,);
}
impl Mapping {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Storage, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = (
        navigation::grant::Owner,
        navigation::grant::Grantor,
        navigation::grant::Grantee,
    );
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Storage, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Model;
impl Entity for Model {
    const NAME: &'static str = "Model";
    type Aspects = (ModelFacts,);
    type Relations = (ModelMember,);
}
impl Model {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Storage, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Member;
impl Entity for Member {
    const NAME: &'static str = "Member";
    type Aspects = (MemberFacts, Option<Calibration>);
    type Relations = (ChildOf, UpstreamOf);
}
impl Member {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Storage, Self)
    }
}
#[derive(Clone, Copy)]
pub struct FirstOutput;
impl Entity for FirstOutput {
    const NAME: &'static str = "FirstOutput";
    type Aspects = (FirstFacts,);
    type Relations = (ChildOfFirst, FirstOf, JoinedOf, ReconciledSourceOf);
}
impl FirstOutput {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Storage, Self)
    }
}
#[derive(Clone, Copy)]
pub struct SecondOutput;
impl Entity for SecondOutput {
    const NAME: &'static str = "SecondOutput";
    type Aspects = (SecondFacts,);
    type Relations = (SecondOf,);
}
impl SecondOutput {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Storage, Self)
    }
}
#[derive(Clone, Copy)]
pub struct ChildOutput;
impl Entity for ChildOutput {
    const NAME: &'static str = "ChildOutput";
    type Aspects = (ChildFacts,);
    type Relations = ();
}
impl ChildOutput {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Storage, Self)
    }
}
pub struct MappingFacts;
impl Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingFactsRow;
}
impl MappingFacts {
    pub fn reference() -> AspectSelection<Storage, Mapping, Self> {
        aspect(Storage, Mapping, Self)
    }
}
pub struct MappingFactsRow {
    pub externalid: WorthQueryExternalPrincipalIdentity,
    pub status: WorthQueryPrincipalMappingStatus,
}
impl Record<MappingFactsRow> for Bindings {
    fn record() -> impl Shape<MappingFactsRow, Self> {
        field("ExternalId", |row: &MappingFactsRow| row.externalid.clone())
            .equality_queryable(true)
            .and("Status", |row: &MappingFactsRow| row.status)
            .equality_queryable(true)
            .build(|externalid, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingFactsRow { externalid, status })
            })
    }
}
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsRow;
}
impl PrincipalFacts {
    pub fn reference() -> AspectSelection<Storage, Principal, Self> {
        aspect(Storage, Principal, Self)
    }
}
pub struct PrincipalFactsRow {
    pub principalid: u64,
}
impl Record<PrincipalFactsRow> for Bindings {
    fn record() -> impl Shape<PrincipalFactsRow, Self> {
        field("PrincipalId", |row: &PrincipalFactsRow| row.principalid)
            .equality_queryable(true)
            .read_only()
            .build(|principalid, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsRow { principalid })
            })
    }
}
pub struct ModelFacts;
impl Aspect for ModelFacts {
    const NAME: &'static str = "ModelFacts";
    type Row = ModelFactsRow;
}
impl ModelFacts {
    pub fn reference() -> AspectSelection<Storage, Model, Self> {
        aspect(Storage, Model, Self)
    }
}
pub struct ModelFactsRow {
    pub modelkey: String,
    pub modelvalue: u64,
}
impl Record<ModelFactsRow> for Bindings {
    fn record() -> impl Shape<ModelFactsRow, Self> {
        field("ModelKey", |row: &ModelFactsRow| row.modelkey.clone())
            .equality_queryable(true)
            .unique()
            .and("ModelValue", |row: &ModelFactsRow| row.modelvalue)
            .equality_queryable(true)
            .build(|modelkey, modelvalue, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ModelFactsRow {
                    modelkey,
                    modelvalue,
                })
            })
    }
}
pub struct MemberFacts;
impl Aspect for MemberFacts {
    const NAME: &'static str = "MemberFacts";
    type Row = MemberFactsRow;
}
impl MemberFacts {
    pub fn reference() -> AspectSelection<Storage, Member, Self> {
        aspect(Storage, Member, Self)
    }
}
pub struct MemberFactsRow {
    pub memberkey: String,
    pub membervalue: u64,
}
impl Record<MemberFactsRow> for Bindings {
    fn record() -> impl Shape<MemberFactsRow, Self> {
        field("MemberKey", |row: &MemberFactsRow| row.memberkey.clone())
            .equality_queryable(true)
            .unique()
            .and("MemberValue", |row: &MemberFactsRow| row.membervalue)
            .equality_queryable(true)
            .build(|memberkey, membervalue, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MemberFactsRow {
                    memberkey,
                    membervalue,
                })
            })
    }
}
pub struct FirstFacts;
impl Aspect for FirstFacts {
    const NAME: &'static str = "FirstFacts";
    type Row = FirstFactsRow;
}
impl FirstFacts {
    pub fn reference() -> AspectSelection<Storage, FirstOutput, Self> {
        aspect(Storage, FirstOutput, Self)
    }
}
pub struct FirstFactsRow {
    pub firstkey: String,
    pub firstvalue: u64,
}
impl Record<FirstFactsRow> for Bindings {
    fn record() -> impl Shape<FirstFactsRow, Self> {
        field("FirstKey", |row: &FirstFactsRow| row.firstkey.clone())
            .equality_queryable(true)
            .unique()
            .and("FirstValue", |row: &FirstFactsRow| row.firstvalue)
            .equality_queryable(true)
            .build(|firstkey, firstvalue, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(FirstFactsRow {
                    firstkey,
                    firstvalue,
                })
            })
    }
}
pub struct ChildFacts;
impl Aspect for ChildFacts {
    const NAME: &'static str = "ChildFacts";
    type Row = ChildFactsRow;
}
impl ChildFacts {
    pub fn reference() -> AspectSelection<Storage, ChildOutput, Self> {
        aspect(Storage, ChildOutput, Self)
    }
}
pub struct ChildFactsRow {
    pub childkey: String,
    pub childvalue: u64,
}
impl Record<ChildFactsRow> for Bindings {
    fn record() -> impl Shape<ChildFactsRow, Self> {
        field("ChildKey", |row: &ChildFactsRow| row.childkey.clone())
            .equality_queryable(true)
            .unique()
            .and("ChildValue", |row: &ChildFactsRow| row.childvalue)
            .equality_queryable(true)
            .build(|childkey, childvalue, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ChildFactsRow {
                    childkey,
                    childvalue,
                })
            })
    }
}
pub struct SecondFacts;
impl Aspect for SecondFacts {
    const NAME: &'static str = "SecondFacts";
    type Row = SecondFactsRow;
}
impl SecondFacts {
    pub fn reference() -> AspectSelection<Storage, SecondOutput, Self> {
        aspect(Storage, SecondOutput, Self)
    }
}
pub struct SecondFactsRow {
    pub secondkey: String,
    pub secondvalue: u64,
}
impl Record<SecondFactsRow> for Bindings {
    fn record() -> impl Shape<SecondFactsRow, Self> {
        field("SecondKey", |row: &SecondFactsRow| row.secondkey.clone())
            .equality_queryable(true)
            .unique()
            .and("SecondValue", |row: &SecondFactsRow| row.secondvalue)
            .equality_queryable(true)
            .build(|secondkey, secondvalue, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SecondFactsRow {
                    secondkey,
                    secondvalue,
                })
            })
    }
}
pub type ExternalId =
    FieldSelection<Storage, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status = FieldSelection<Storage, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalId = FieldSelection<Storage, Principal, PrincipalFacts, u64>;
pub type ModelKey = FieldSelection<Storage, Model, ModelFacts, String>;
pub type ModelValue = FieldSelection<Storage, Model, ModelFacts, u64>;
pub type MemberKey = FieldSelection<Storage, Member, MemberFacts, String>;
pub type MemberValue = FieldSelection<Storage, Member, MemberFacts, u64>;
pub type FirstKey = FieldSelection<Storage, FirstOutput, FirstFacts, String>;
pub type FirstValue = FieldSelection<Storage, FirstOutput, FirstFacts, u64>;
pub type SecondKey = FieldSelection<Storage, SecondOutput, SecondFacts, String>;
pub type SecondValue = FieldSelection<Storage, SecondOutput, SecondFacts, u64>;
pub type ChildKey = FieldSelection<Storage, ChildOutput, ChildFacts, String>;
pub type ChildValue = FieldSelection<Storage, ChildOutput, ChildFacts, u64>;
pub struct ChildOf;
impl Relation for ChildOf {
    const NAME: &'static str = "ChildOf";
    type To = ChildOutput;
}
impl ChildOf {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Member, ChildOutput> {
        relation(Storage, Member, Self)
    }
}
pub struct ChildOfFirst;
impl Relation for ChildOfFirst {
    const NAME: &'static str = "ChildOfFirst";
    type To = ChildOutput;
}
impl ChildOfFirst {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, FirstOutput, ChildOutput> {
        relation(Storage, FirstOutput, Self)
    }
}
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Mapping, Principal> {
        relation(Storage, Mapping, Self)
    }
}
pub struct ModelMember;
impl Relation for ModelMember {
    const NAME: &'static str = "ModelMember";
    type To = Member;
}
impl ModelMember {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Model, Member> {
        relation(Storage, Model, Self)
    }
}
pub struct UpstreamOf;
impl Relation for UpstreamOf {
    const NAME: &'static str = "UpstreamOf";
    type To = Model;
}
impl UpstreamOf {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Member, Model> {
        relation(Storage, Member, Self)
    }
}
pub struct FirstOf;
impl Relation for FirstOf {
    const NAME: &'static str = "FirstOf";
    type To = Model;
}
impl FirstOf {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, FirstOutput, Model> {
        relation(Storage, FirstOutput, Self)
    }
}
pub struct SecondOf;
impl Relation for SecondOf {
    const NAME: &'static str = "SecondOf";
    type To = Model;
}
impl SecondOf {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, SecondOutput, Model> {
        relation(Storage, SecondOutput, Self)
    }
}
