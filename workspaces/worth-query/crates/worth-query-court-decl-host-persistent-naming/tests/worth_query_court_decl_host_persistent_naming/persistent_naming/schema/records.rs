//! Native fragment membership and original record shapes for this fixture.
use super::*;
use worth_query_decl::facade::application_naming::{
    EntityNameSubject, NamedTarget, OwnedThrough, OwnerViaReverse, RelationNameSubject, Unowned,
    WorthQueryName, WorthQueryNamedKind,
};
use worth_query_decl::facade::WorthQueryRecord;
#[derive(Clone, Copy)]
pub struct Storage;
impl Fragment for Storage {
    const NAME: &'static str = "storage";
    type Bindings = Bindings;
    type Members = (Mapping, Principal, Model, Member);
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
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Bindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<String> for Bindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = true;
}
/// A hand-written binding set stores held names with the stock codecs.
impl<K: WorthQueryNamedKind> Scalar<WorthQueryName<K>> for Bindings {
    type Binding = WorthQueryNameApplicationValueBinding<K>;
}
impl<K: WorthQueryNamedKind> Scalar<NamedTarget<K>> for Bindings {
    type Binding = NamedTargetApplicationValueBinding<K>;
}
/// A member is owned by the model that lists it; a seam has no owner.
impl WorthQueryNamedKind for Member {
    type Subject = EntityNameSubject;
    type OwnerScope = OwnedThrough<OwnerViaReverse<ModelMember, Model>>;
}
impl WorthQueryNamedKind for Seam {
    type Subject = RelationNameSubject;
    type OwnerScope = Unowned;
}
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
    type Relations = (ModelMember, Reference);
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
    type Aspects = (MemberFacts,);
    type Relations = (Child, Seam);
}
impl Member {
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
/// The model's facts, derived: a model may hold a member by persistent name
/// as an ordinary column, a `NamedTarget` and a bare `WorthQueryName`.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Bindings, aspect = ModelFacts, fragment = Storage, entity = Model)]
pub struct ModelFactsRow {
    #[worth_query(column = "ModelKey", equality, unique)]
    pub modelkey: String,
    #[worth_query(column = "ModelValue", equality)]
    pub modelvalue: u64,
    pub held_target: Option<NamedTarget<Member>>,
    pub held_anchor: Option<WorthQueryName<Member>>,
}
impl ModelFacts {
    pub fn reference() -> AspectSelection<Storage, Model, Self> {
        aspect(Storage, Model, Self)
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
pub type ExternalId =
    FieldSelection<Storage, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status = FieldSelection<Storage, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalId = FieldSelection<Storage, Principal, PrincipalFacts, u64>;
pub type MemberKey = FieldSelection<Storage, Member, MemberFacts, String>;
pub type MemberValue = FieldSelection<Storage, Member, MemberFacts, u64>;
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
pub struct Child;
impl Relation for Child {
    const NAME: &'static str = "Child";
    type To = Member;
}
impl Child {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Member, Member> {
        relation(Storage, Member, Self)
    }
}
/// A persistently named relation between members (Slice 5C output naming).
pub struct Seam;
impl Relation for Seam {
    const NAME: &'static str = "Seam";
    type To = Member;
}
impl Seam {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Member, Member> {
        relation(Storage, Member, Self)
    }
}
pub struct Reference;
impl Relation for Reference {
    const NAME: &'static str = "Reference";
    type To = Member;
}
impl Reference {
    pub fn binding() -> ApplicationRelationRef<Schema, Self, Model, Member> {
        relation(Storage, Model, Self)
    }
}
