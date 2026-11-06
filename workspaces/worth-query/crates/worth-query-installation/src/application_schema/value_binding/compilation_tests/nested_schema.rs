//! A real declaration supplies one field codec to installation admission.
use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
pub(super) struct NestedSchema;
impl Fragment for NestedSchema {
    const NAME: &'static str = "NestedSchema";
    type Bindings = Bindings;
    type Members = (Entity,);
}
impl ApplicationSchema for NestedSchema {
    const OWNER: &'static str = "NestedSchema";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
pub(super) struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<Vec<String>> for Bindings {
    type Binding = NestedBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = false;
}
pub(super) struct Entity;
impl fragment::Entity for Entity {
    const NAME: &'static str = "Entity";
    type Aspects = (Aspect,);
    type Relations = ();
}

pub(super) struct Aspect;
impl fragment::Aspect for Aspect {
    const NAME: &'static str = "Aspect";
    type Row = Row;
}
pub(super) struct Row {
    value: Vec<String>,
}
impl fragment::Record<Row> for Bindings {
    fn record() -> impl Shape<Row, Self> {
        field("Field", |row: &Row| row.value.clone()).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row { value })
            },
        )
    }
}
pub(super) type Field = FieldSelection<NestedSchema, Entity, Aspect, Vec<String>>;
