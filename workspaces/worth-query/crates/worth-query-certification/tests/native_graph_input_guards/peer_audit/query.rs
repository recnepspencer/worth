//! A parent-affine governed query observes actual attachment, actor and computed revision.
use super::super::child_actions::{Detail, DetailFacts, DetailLabel};
use super::{super::schema::*, schema::*};
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_portable_type, worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};
#[derive(Clone)]
pub struct Read(pub String);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value {
    pub name: String,
    pub revision: u64,
    pub actor: u64,
    pub detail: Option<String>,
    pub entity: worth_relational::facade::identity::EntityId,
}
worth_query_portable_type!(Value=>"worth.query.peer-journal.result.v1");
worth_query_structured_value_binding!(pub Input for Read {identity:"worth.query.peer-journal.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Value {identity:"worth.query.peer-journal.result.v1"});
worth_query_structured_value_binding!(pub Parameters for () {identity:"worth.query.peer-journal.parameters.v1"});
worth_query_application_query!(pub Query for Schema,identity "PeerJournal",parameters Parameters,result ResultBinding,scope Parent=>"Parent",name "parent_journal");
macro_rules! field {
    ($slot:ident,$name:ident,$field:ty,$value:ty,$identity:literal)=>{
        pub struct $slot;worth_query_portable_type!($slot=>$identity);
        fn $name()->ApplicationQueryResultFieldRef<Query,$slot,Schema,Journal,JournalFacts,$field,$value,ReadOnly,EqualityPredicate,NoApplicationUnit>{
            ApplicationQueryResultFieldRef::new(stringify!($name),<$field>::reference())
        }
    }
}
field!(
    Name,
    name,
    JournalName,
    String,
    "worth.query.peer-journal.name.v1"
);
field!(
    Revision,
    revision,
    JournalRevision,
    u64,
    "worth.query.peer-journal.revision.v1"
);
field!(
    Actor,
    actor,
    JournalActor,
    u64,
    "worth.query.peer-journal.actor.v1"
);
pub struct DetailSlot;
worth_query_portable_type!(DetailSlot=>"worth.query.peer-journal.detail-slot.v1");
worth_query_structured_value_binding!(pub DetailResult for String {identity:"worth.query.peer-journal.detail-result.v1"});
pub struct Label;
worth_query_portable_type!(Label=>"worth.query.peer-journal.detail-label.v1");
fn label() -> ApplicationQueryResultFieldRef<
    Query,
    Label,
    Schema,
    Detail,
    DetailFacts,
    DetailLabel,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("label", DetailLabel::reference())
}
fn details() -> ApplicationQueryResultRelationRef<
    Query,
    DetailSlot,
    Schema,
    JournalDetails,
    Journal,
    Detail,
    ForwardResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::forward_optional("detail", JournalDetails::reference())
}
pub fn definition() -> ApplicationQueryDefinition<Schema, Query, (), Value, Parent> {
    ApplicationQueryDefinitionBuilder::declare(Query::reference()).root(Journal::reference()).scope(Parent::reference())
        .result_shape(ApplicationQueryResultShapeBuilder::<Schema,Query,Journal,Value,ResultBinding>::new(Journal::reference()).field(name()).field(revision()).field(actor())
            .relation(details(),ApplicationQueryResultShapeBuilder::<Schema,Query,Detail,String,DetailResult>::new(Detail::reference()).field(label())).build())
        .cardinality(ApplicationQueryCardinality::Many)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(2,2,4))
        .disclosure(ApplicationQueryDisclosureContract::public()).basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot()).public()
        .root_path(ApplicationQueryRootPath::from(Parent::reference()).forward(ParentJournal::reference())).build().unwrap()
}
impl WorthQueryApplicationProjection<Schema, Query> for Value {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            name: row.field(name())?,
            revision: row.field(revision())?,
            actor: row.field(actor())?,
            detail: row
                .optional(details())?
                .map(|detail| detail.field(label()))
                .transpose()?,
            entity: row.entity_id(),
        })
    }
}
fn scope(read: Read) -> String {
    read.0
}
worth_query_query_binding!(pub ReadBinding for Read,schema Schema,identity "worth.query.peer-journal.binding.v1",input Input,query Query,parameters Parameters=>|_|ApplicationQueryParameterSet::new(),result ResultBinding,
principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
scope Parent,ParentFacts,ParentIdentity,String,ReadOnly,NoApplicationUnit,field ParentIdentity::reference(),value scope,limits results 20);
