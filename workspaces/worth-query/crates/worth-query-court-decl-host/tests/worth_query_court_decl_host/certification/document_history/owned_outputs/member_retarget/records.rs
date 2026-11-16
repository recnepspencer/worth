//! Authored members are distinct from this world's producer-owned Member rows.
use super::*;
pub struct InputMember;
impl Entity for InputMember {
    const NAME: &'static str = "InputMember";
    type Aspects = (InputFacts,);
    type Relations = (InputOwner,);
}
impl InputMember {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(Storage, Self)
    }
}
pub struct InputFacts;
impl Aspect for InputFacts {
    const NAME: &'static str = "InputFacts";
    type Row = Row;
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Row {
    pub key: String,
    pub value: u64,
}
impl Record<Row> for Bindings {
    fn record() -> impl Shape<Row, Self> {
        field("InputKey", |row: &Row| row.key.clone())
            .equality_queryable(true)
            .unique()
            .and("InputValue", |row: &Row| row.value)
            .equality_queryable(true)
            .build(|key, value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row { key, value })
            })
    }
}
pub type InputKey = FieldSelection<Storage, InputMember, InputFacts, String>;
pub type InputValue = FieldSelection<Storage, InputMember, InputFacts, u64>;
pub struct InputOwner;
impl Relation for InputOwner {
    const NAME: &'static str = "InputOwner";
    type To = Model;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, None);
}
impl InputOwner {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, InputMember, Model> {
        relation(Storage, InputMember, Self)
    }
}
pub struct Family;
pub struct ConsumedInput;
impl Relation for ConsumedInput {
    const NAME: &'static str = "ConsumedInput";
    type To = InputMember;
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::CascadeDeleteRelations;
}
impl ConsumedInput {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, FirstOutput, InputMember> {
        relation(Storage, FirstOutput, Self)
    }
}
impl ApplicationAuthoredCorrectionFamily<Schema> for Family {
    type Entity = InputMember;
    const IDENTITY: &'static str = "owned-outputs.input-member-family";
    fn logical_key() -> ApplicationAuthoredLogicalKey<Schema, Self::Entity> {
        ApplicationAuthoredLogicalKey::from_field(InputKey::at("InputKey"))
    }
}
