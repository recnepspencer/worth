//! Domain rows, binding ownership, and tuple membership for this fixture.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub(super) struct CardinalitySchema;
impl Fragment for CardinalitySchema {
    const NAME: &'static str = "CardinalitySchema";
    type Bindings = Bindings;
    type Members = (Parent, Child, OptionalChildQuery, ManyChildrenQuery);
}
impl ApplicationSchema for CardinalitySchema {
    const OWNER: &'static str = "application_query_cardinality_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl CardinalitySchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub(super) struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
pub(super) struct Parent;
impl Entity for Parent {
    const NAME: &'static str = "Parent";
    type Aspects = (ParentFacts,);
    type Relations = (ParentChild,);
}
impl Parent {
    pub fn reference() -> ApplicationEntityRef<CardinalitySchema, Self> {
        entity(CardinalitySchema, Self)
    }
}
pub(super) struct Child;
impl Entity for Child {
    const NAME: &'static str = "Child";
    type Aspects = (ChildFacts,);
    type Relations = ();
}
impl Child {
    pub fn reference() -> ApplicationEntityRef<CardinalitySchema, Self> {
        entity(CardinalitySchema, Self)
    }
}
pub(super) struct ParentFacts;
impl Aspect for ParentFacts {
    const NAME: &'static str = "ParentFacts";
    type Row = ParentFactsValue;
}

pub(super) struct ParentFactsValue {
    pub parent_id: u64,
}
impl Record<ParentFactsValue> for Bindings {
    fn record() -> impl Shape<ParentFactsValue, Self> {
        field("ParentId", |row: &ParentFactsValue| row.parent_id)
            .read_only()
            .build(|parent_id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ParentFactsValue { parent_id })
            })
    }
}
pub(super) struct ChildFacts;
impl Aspect for ChildFacts {
    const NAME: &'static str = "ChildFacts";
    type Row = ChildFactsValue;
}

pub(super) struct ChildFactsValue {
    pub child_id: u64,
}
impl Record<ChildFactsValue> for Bindings {
    fn record() -> impl Shape<ChildFactsValue, Self> {
        field("ChildId", |row: &ChildFactsValue| row.child_id)
            .read_only()
            .build(|child_id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ChildFactsValue { child_id })
            })
    }
}
pub(super) type ParentId = FieldSelection<CardinalitySchema, Parent, ParentFacts, u64>;
pub(super) type ChildId = FieldSelection<CardinalitySchema, Child, ChildFacts, u64>;
pub(super) struct ParentChild;
impl Relation for ParentChild {
    const NAME: &'static str = "ParentChild";
    type To = Child;
}
impl ParentChild {
    pub fn reference() -> ApplicationRelationRef<CardinalitySchema, Self, Parent, Child> {
        relation(CardinalitySchema, Parent, Self)
    }
}
impl Registration<CardinalitySchema> for OptionalChildQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<CardinalitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<CardinalitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(query_definition(
            Self::reference(),
            ApplicationQueryCardinality::OptionalOne,
        )))
    }
}
impl Registration<CardinalitySchema> for ManyChildrenQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<CardinalitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<CardinalitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(query_definition(
            Self::reference(),
            ApplicationQueryCardinality::Many,
        )))
    }
}
