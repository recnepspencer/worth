//! The fixture's only application action emits one actual Bank rail payload.
use super::{operation::*, rejection::*, schema::*};
use worth_query_decl::facade::application_program::*;
pub struct RailHomeProgram;
pub struct NoticeCompletion;
impl ApplicationActionDelivery<RailHomeSchema, NotifyRailBinding> for NoticeCompletion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.bank.rail-home.completion.v1";
}
pub struct UnservedCompletion;
impl ApplicationActionDelivery<RailHomeSchema, UnservedNoticeBinding> for UnservedCompletion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.bank.rail-home.unserved-completion.v1";
}
pub struct RailFeature;
impl ApplicationFeature<RailHomeSchema> for RailFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.bank.rail-home.feature.v1";
}
impl ApplicationProgramDefinition<RailHomeSchema> for RailHomeProgram {
    type Contributions = (RailSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.bank.rail-home.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<RailHomeSchema, RailFeature>()
                .action::<NotifyRailBinding, NoticeCompletion>()
                .finish_action()
                .action::<UnservedNoticeBinding, UnservedCompletion>()
                .finish_action()
                .finish(),
        ]
    }
}
