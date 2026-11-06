//! A real declaration supplies one field codec to installation admission.
use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
pub(super) struct OpaqueSchema;
impl Fragment for OpaqueSchema {
    const NAME: &'static str = "OpaqueSchema";
    type Bindings = Bindings;
    type Members = (OpaqueEntity,);
}
impl ApplicationSchema for OpaqueSchema {
    const OWNER: &'static str = "OpaqueSchema";
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
    type Binding = OpaqueBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = false;
}
pub(super) struct OpaqueEntity;
impl fragment::Entity for OpaqueEntity {
    const NAME: &'static str = "OpaqueEntity";
    type Aspects = (OpaqueAspect,);
    type Relations = ();
}

pub(super) struct OpaqueAspect;
impl fragment::Aspect for OpaqueAspect {
    const NAME: &'static str = "OpaqueAspect";
    type Row = Row;
}
pub(super) struct Row {
    value: String,
}
impl fragment::Record<Row> for Bindings {
    fn record() -> impl Shape<Row, Self> {
        field("OpaqueField", |row: &Row| row.value.clone()).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row { value })
            },
        )
    }
}
