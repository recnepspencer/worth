//! Ordinary source-completion delivery; no retained SourceOnly owner is constructed.
use super::{
    operation::NoticeBinding,
    schema::{Schema, Surface},
};
use worth_query_decl::facade::application_program::*;
pub struct Program;
pub struct Completion;
impl ApplicationActionDelivery<Schema, NoticeBinding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.compensation-original.completion.v1";
}
struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.compensation-original.feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Surface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.compensation-original.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Feature>()
            .action::<NoticeBinding, Completion>()
            .finish_action()
            .finish()]
    }
}
