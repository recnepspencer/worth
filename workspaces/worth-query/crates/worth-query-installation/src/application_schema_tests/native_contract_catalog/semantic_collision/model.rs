//! Distinct entity owners reuse a row and its semantic aspect name.
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct SourceSchema;
impl ApplicationSchema for SourceSchema {
    const OWNER: &'static str = "native-collision";
    const NAME: &'static str = "SourceSchema";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::from_fragment(CollisionRecords)?.build()
    }
}
pub struct CollisionRecords;
impl Fragment for CollisionRecords {
    const NAME: &'static str = "collisions";
    type Bindings = Bindings;
    type Members = (FirstNamedEntity, SecondNamedEntity, FieldEntity);
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
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = true;
}
pub struct FirstNamedEntity;
impl Entity for FirstNamedEntity {
    const NAME: &'static str = "FirstNamedEntity";
    type Aspects = (NamedAspect,);
    type Relations = ();
}
pub struct SecondNamedEntity;
impl Entity for SecondNamedEntity {
    const NAME: &'static str = "SecondNamedEntity";
    type Aspects = (NamedAspect,);
    type Relations = ();
}
pub struct FieldEntity;
impl Entity for FieldEntity {
    const NAME: &'static str = "FieldEntity";
    type Aspects = (FieldAspect,);
    type Relations = ();
}
pub struct NamedAspect;
impl Aspect for NamedAspect {
    const NAME: &'static str = "SameAspect";
    type Row = Row;
}
pub struct FieldAspect;
impl Aspect for FieldAspect {
    const NAME: &'static str = "FieldAspect";
    type Row = Row;
}
pub struct Row {
    value: u64,
}
impl fragment::Record<Row> for Bindings {
    fn record() -> impl Shape<Row, Self> {
        field("SameField", |row: &Row| row.value).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row { value })
            },
        )
    }
}
pub type FirstField = FieldSelection<CollisionRecords, FirstNamedEntity, NamedAspect, u64>;
pub type SecondField = FieldSelection<CollisionRecords, SecondNamedEntity, NamedAspect, u64>;
