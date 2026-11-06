//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct CompleteModel;
impl fragment::Fragment for CompleteModel {
    const NAME: &'static str = "complete_model";
    type Bindings = CompleteModelBindings;
    type Members = (CompleteEntity,);
}
pub struct CompleteModelBindings;
impl fragment::BindingSet for CompleteModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<String> for CompleteModelBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
pub struct CompleteEntity;
impl fragment::Entity for CompleteEntity {
    const NAME: &'static str = "CompleteEntity";
    type Aspects = (Profile,);
    type Relations = ();
}
impl CompleteEntity {
    pub fn reference() -> ApplicationEntityRef<CompleteSchema, Self> {
        entity(CompleteModel, Self)
    }
}
pub struct Profile;
impl fragment::Aspect for Profile {
    const NAME: &'static str = "Profile";
    type Row = ProfileValue;
}
impl Profile {
    pub fn reference() -> AspectSelection<CompleteModel, CompleteEntity, Self> {
        aspect(CompleteModel, CompleteEntity, Self)
    }
}
pub struct ProfileValue {
    pub name: String,
}
impl fragment::Record<ProfileValue> for CompleteModelBindings {
    fn record() -> impl fragment::Shape<ProfileValue, Self> {
        fragment::field("Name", |row: &ProfileValue| row.name.clone()).build(
            |name, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ProfileValue { name })
            },
        )
    }
}
type Name = FieldSelection<CompleteModel, CompleteEntity, Profile, String>;
