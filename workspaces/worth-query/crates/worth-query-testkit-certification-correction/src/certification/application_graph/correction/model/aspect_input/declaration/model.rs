//! Required and optional parents are distinct declarations with the same row meaning.
use super::*;
pub(super) struct OptionalModel;
impl Fragment for OptionalModel {
    const NAME: &'static str = "input_removal";
    type Bindings = NumericModelBindings;
    type Members = (OptionalNumber,);
}
pub(super) struct RequiredModel;
impl Fragment for RequiredModel {
    const NAME: &'static str = <OptionalModel as Fragment>::NAME;
    type Bindings = NumericModelBindings;
    type Members = (RequiredNumber,);
}
pub(super) struct OptionalNumber;
impl Entity for OptionalNumber {
    const NAME: &'static str = <Number as Entity>::NAME;
    type Aspects = (Option<InputFacts>,);
    type Relations = ();
}
pub(super) struct RequiredNumber;
impl Entity for RequiredNumber {
    const NAME: &'static str = <Number as Entity>::NAME;
    type Aspects = (InputFacts,);
    type Relations = ();
}
