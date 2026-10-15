use worth_query_host::facade::declaration::application_program::{
    ApplicationActionDelivery, ApplicationCommitBoundary, ApplicationFeature,
    ApplicationFeatureInputLeaf, ApplicationFeatureSpec, ApplicationLocalRuleRef,
    ApplicationNoOutputGraph, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationProgramOutputs, ApplicationRetainedCommit,
    ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList, ValidatedApplicationProgram,
};

use crate::declaration::{
    WorthUiApplicationSchema, WorthUiRecordContribution, WorthUiStatusActionBinding,
    WorthUiStatusIntegrity, WorthUiStatusUpdateBinding,
};

pub(super) struct WorthUiStatusProgram;
pub(super) struct WorthUiStatusFeature;

pub(super) struct WorthUiStatusUpdateDelivery;
impl ApplicationActionDelivery<WorthUiApplicationSchema, WorthUiStatusUpdateBinding>
    for WorthUiStatusUpdateDelivery
{
    type Mode = ApplicationRetainedCommit;
    const IDENTITY: &'static str = "worth.ui.status-update.retained.v1";
}

pub(super) struct WorthUiStatusActionDelivery;
impl ApplicationActionDelivery<WorthUiApplicationSchema, WorthUiStatusActionBinding>
    for WorthUiStatusActionDelivery
{
    type Mode = ApplicationRetainedCommit;
    const IDENTITY: &'static str = "worth.ui.status-action.retained.v1";
}

impl ApplicationFeature<WorthUiApplicationSchema> for WorthUiStatusFeature {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.ui.status-feature.v1";
}

impl ApplicationProgramDefinition<WorthUiApplicationSchema> for WorthUiStatusProgram {
    type Contributions = (WorthUiRecordContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationLocalRuleRef<
                WorthUiApplicationSchema,
                WorthUiStatusFeature,
                WorthUiStatusIntegrity,
            >,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.ui.status-program.v2");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<WorthUiApplicationSchema, WorthUiStatusFeature>()
                .action::<WorthUiStatusUpdateBinding, WorthUiStatusUpdateDelivery>()
                .finish_action()
                .action::<WorthUiStatusActionBinding, WorthUiStatusActionDelivery>()
                .finish_action()
                .finish(),
        ]
    }
}

pub(super) fn validated_status_program() -> Result<
    ValidatedApplicationProgram<WorthUiApplicationSchema, WorthUiStatusProgram>,
    worth_query_host::facade::declaration::application_program::ApplicationProgramValidationDenial,
> {
    ApplicationProgramAuthoring::<WorthUiApplicationSchema, WorthUiStatusProgram>::begin()
        .validated_program()
}
