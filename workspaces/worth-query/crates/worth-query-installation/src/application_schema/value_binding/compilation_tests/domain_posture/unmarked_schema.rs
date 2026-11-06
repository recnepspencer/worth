//! A real declaration supplies one field codec to installation admission.
use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
pub(super) struct UnmarkedSchema;
impl Fragment for UnmarkedSchema {
    const NAME: &'static str = "UnmarkedSchema";
    type Bindings = Bindings;
    type Members = (UnmarkedEntity,);
}
impl ApplicationSchema for UnmarkedSchema {
    const OWNER: &'static str = "UnmarkedSchema";
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
impl Scalar<String> for Bindings {
    type Binding = Unmarked;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = false;
}
pub(super) struct UnmarkedEntity;
impl fragment::Entity for UnmarkedEntity {
    const NAME: &'static str = "UnmarkedEntity";
    type Aspects = (UnmarkedAspect,);
    type Relations = ();
}

pub(super) struct UnmarkedAspect;
impl fragment::Aspect for UnmarkedAspect {
    const NAME: &'static str = "UnmarkedAspect";
    type Row = Row;
}
pub(super) struct Row {
    value: String,
}
impl fragment::Record<Row> for Bindings {
    fn record() -> impl Shape<Row, Self> {
        field("UnmarkedField", |row: &Row| row.value.clone()).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row { value })
            },
        )
    }
}
