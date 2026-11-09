//! A real declaration supplies one field codec to installation admission.
use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
pub(super) struct RawEquality;
impl Fragment for RawEquality {
    const NAME: &'static str = "RawEquality";
    type Bindings = Bindings;
    type Members = (RawEntity,);
}
impl ApplicationSchema for RawEquality {
    const OWNER: &'static str = "RawEquality";
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
    type Binding = RawBytes;
    const WRITABLE: bool = false;
}
pub(super) struct RawEntity;
impl fragment::Entity for RawEntity {
    const NAME: &'static str = "RawEntity";
    type Aspects = (RawAspect,);
    type Relations = ();
}

pub(super) struct RawAspect;
impl fragment::Aspect for RawAspect {
    const NAME: &'static str = "RawAspect";
    type Row = Row;
}
pub(super) struct Row {
    value: String,
}
impl fragment::Record<Row> for Bindings {
    fn record() -> impl Shape<Row, Self> {
        field("RawField", |row: &Row| row.value.clone())
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row { value })
            })
    }
}
pub(super) type RawField = FieldSelection<RawEquality, RawEntity, RawAspect, String>;
