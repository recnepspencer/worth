//! A real declaration supplies one field codec to installation admission.
use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
pub(super) struct LegacySchema;
impl Fragment for LegacySchema {
    const NAME: &'static str = "LegacySchema";
    type Bindings = Bindings;
    type Members = (LegacyEntity,);
}
impl ApplicationSchema for LegacySchema {
    const OWNER: &'static str = "LegacySchema";
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
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = false;
}
pub(super) struct LegacyEntity;
impl fragment::Entity for LegacyEntity {
    const NAME: &'static str = "LegacyEntity";
    type Aspects = (LegacyAspect,);
    type Relations = ();
}

pub(super) struct LegacyAspect;
impl fragment::Aspect for LegacyAspect {
    const NAME: &'static str = "LegacyAspect";
    type Row = Row;
}
pub(super) struct Row {
    value: u64,
}
impl fragment::Record<Row> for Bindings {
    fn record() -> impl Shape<Row, Self> {
        field("LegacyField", |row: &Row| row.value).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row { value })
            },
        )
    }
}
