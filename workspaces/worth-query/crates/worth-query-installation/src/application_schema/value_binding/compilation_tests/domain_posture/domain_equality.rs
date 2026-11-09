//! A real declaration supplies one field codec to installation admission.
use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
pub(super) struct DomainEquality;
impl Fragment for DomainEquality {
    const NAME: &'static str = "DomainEquality";
    type Bindings = Bindings;
    type Members = (DomainEntity,);
}
impl ApplicationSchema for DomainEquality {
    const OWNER: &'static str = "DomainEquality";
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
}
pub(super) struct DomainEntity;
impl fragment::Entity for DomainEntity {
    const NAME: &'static str = "DomainEntity";
    type Aspects = (DomainAspect,);
    type Relations = ();
}

pub(super) struct DomainAspect;
impl fragment::Aspect for DomainAspect {
    const NAME: &'static str = "DomainAspect";
    type Row = Row;
}
pub(super) struct Row {
    value: String,
}
impl fragment::Record<Row> for Bindings {
    fn record() -> impl Shape<Row, Self> {
        field("DomainField", |row: &Row| row.value.clone())
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row { value })
            })
    }
}
pub(super) type DomainField = FieldSelection<DomainEquality, DomainEntity, DomainAspect, String>;
