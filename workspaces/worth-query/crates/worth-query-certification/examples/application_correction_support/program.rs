//! A real source action with ordinary completion and no required output root.
use super::{edit::EditBinding, other_edit::OtherBinding, schema::*};
use worth_query_host::facade::declaration::application_program::*;
pub struct Program;
pub struct NumericFeature;
pub struct Completion;
pub struct OtherCompletion;
pub struct PrincipalCompletion;
impl ApplicationActionDelivery<NumericSchema, super::principal_status::Binding>
    for PrincipalCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-correction.principal-completion.v1";
}
impl ApplicationActionDelivery<NumericSchema, OtherBinding> for OtherCompletion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-correction.other-completion.v1";
}
impl ApplicationFeature<NumericSchema> for NumericFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "numeric-correction.feature.v1";
}
impl ApplicationActionDelivery<NumericSchema, EditBinding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-correction.completion.v1";
}
pub type Action = ApplicationActionReference<
    NumericSchema,
    ApplicationRootComposition,
    NumericFeature,
    EditBinding,
    Completion,
>;
impl ApplicationProgramDefinition<NumericSchema> for Program {
    type Contributions = (NumericContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<NumericSchema, NumericFeature>()
                .action::<EditBinding, Completion>()
                .finish_action()
                .action::<OtherBinding, OtherCompletion>()
                .finish_action()
                .action::<super::principal_status::Binding, PrincipalCompletion>()
                .finish_action()
                .finish(),
        ]
    }
}
