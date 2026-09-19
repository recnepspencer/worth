use worth_query_host::facade::declaration::application_program::{
    ApplicationCommitBoundary, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureSpec, ApplicationLocalRuleRef, ApplicationNoOutputGraph,
    ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramIdentity,
    ApplicationProgramOutputs, ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList,
    ValidatedApplicationProgram,
};

use crate::declaration::{
    WorthUiApplicationSchema, WorthUiRecordContribution, WorthUiStatusActionBinding,
    WorthUiStatusIntegrity, WorthUiStatusUpdateBinding,
};

pub(super) struct WorthUiStatusProgram;
pub(super) struct WorthUiStatusFeature;

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
                .mutation::<WorthUiStatusUpdateBinding>()
                .mutation::<WorthUiStatusActionBinding>()
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
