//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct NativeModel;
impl fragment::Fragment for NativeModel {
    const NAME: &'static str = "native_model";
    type Bindings = NativeModelBindings;
    type Members = (NativeEntity,);
}
pub struct NativeModelBindings;
impl fragment::BindingSet for NativeModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for NativeModelBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<String> for NativeModelBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = false;
}
pub struct NativeEntity;
impl fragment::Entity for NativeEntity {
    const NAME: &'static str = "NativeEntity";
    type Aspects = (Profile,);
    type Relations = ();
}
impl NativeEntity {
    pub fn reference() -> ApplicationEntityRef<NativeSchema, Self> {
        entity(NativeModel, Self)
    }
}
pub struct Profile;
impl fragment::Aspect for Profile {
    const NAME: &'static str = "Profile";
    type Row = ProfileValue;
}
impl Profile {
    pub fn reference() -> AspectSelection<NativeModel, NativeEntity, Self> {
        aspect(NativeModel, NativeEntity, Self)
    }
}
pub struct ProfileValue {
    pub age: u64,
    pub tag: String,
}
impl fragment::Record<ProfileValue> for NativeModelBindings {
    fn record() -> impl fragment::Shape<ProfileValue, Self> {
        fragment::field("Age", |row: &ProfileValue| row.age)
            .and("Tag", |row: &ProfileValue| row.tag.clone())
            .build(|age, tag, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ProfileValue { age, tag })
            })
    }
}
type Age = FieldSelection<NativeModel, NativeEntity, Profile, u64>;
type Tag = FieldSelection<NativeModel, NativeEntity, Profile, String>;
