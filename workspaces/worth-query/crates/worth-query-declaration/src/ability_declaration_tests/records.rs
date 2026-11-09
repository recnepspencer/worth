//! One record model owns field lowering and reusable scalar bindings.
use crate::facade::application_schema::{fragment, ValueDecodeAdmission};
pub struct AbilityModel;
impl fragment::Fragment for AbilityModel {
    const NAME: &'static str = "ability_model";
    type Bindings = AbilityModelBindings;
    type Members = (Account,);
}
pub struct AbilityModelBindings;
impl fragment::BindingSet for AbilityModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
pub struct Account;
impl fragment::Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = ();
    type Relations = ();
}
