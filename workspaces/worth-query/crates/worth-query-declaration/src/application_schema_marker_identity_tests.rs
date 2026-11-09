use super::{Aspect, Entity};
use super::{IdentifierEntity, ProgramEntity};

use crate::application_schema::fragment;
pub(super) struct Fixture;
impl fragment::Fragment for Fixture {
    const NAME: &'static str = "fixture";
    type Bindings = Bindings;
    type Members = (Entity,);
}
pub(super) struct Bindings;
impl fragment::BindingSet for Bindings {
    type Work<'a> = crate::application_schema::ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut crate::application_schema::ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for Bindings {
    type Binding = crate::application_schema::U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl fragment::Entity for Entity {
    const NAME: &'static str = "Entity";
    type Aspects = (Aspect,);
    type Relations = ();
}
impl fragment::Aspect for Aspect {
    const NAME: &'static str = "Aspect";
    type Row = FixtureRow;
}
pub(super) fn selected_field() -> fragment::FieldSelection<Fixture, Entity, Aspect, u64> {
    fragment::FieldSelection::at("Field")
}

impl fragment::Entity for ProgramEntity {
    const NAME: &'static str = "ProgramEntity";
    type Aspects = ();
    type Relations = ();
}
pub(super) struct ProgramFixture;
impl fragment::Fragment for ProgramFixture {
    const NAME: &'static str = "program";
    type Bindings = fragment::NoBindings;
    type Members = (ProgramEntity,);
}
impl fragment::Entity for IdentifierEntity {
    const NAME: &'static str = "IdentifierEntity";
    type Aspects = ();
    type Relations = ();
}
pub(super) struct Identifiers;
impl fragment::Fragment for Identifiers {
    const NAME: &'static str = "identifiers";
    type Bindings = fragment::NoBindings;
    type Members = (IdentifierEntity,);
}

pub(super) struct FixtureRow {
    value: u64,
}
impl fragment::Record<FixtureRow> for Bindings {
    fn record() -> impl fragment::Shape<FixtureRow, Self> {
        fragment::field("Field", |row: &FixtureRow| row.value)
            .equality_queryable(true)
            .build(
                |value, _: &mut crate::application_schema::ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(FixtureRow { value })
                },
            )
    }
}
