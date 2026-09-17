use worth_query_host::facade::declaration::application_program::{
    ApplicationActionLeaf, ApplicationActionList, ApplicationCommitBoundary,
    ApplicationConditionalOperationActionRef, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureLeaf, ApplicationFeatureList, ApplicationFeatureRef, ApplicationLocalRuleRef,
    ApplicationNoOutputGraph, ApplicationOperationActionRef, ApplicationProgramAuthoring,
    ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationRuleAt,
    ApplicationRuleLeaf, ApplicationRuleList, ValidatedApplicationProgram,
};

use super::schema::{
    AmendTemporal, AmendTemporalAndPublishDefinition, ExecuteTemporal, RevokeTemporalPrincipal,
    TemporalHostContribution, TemporalHostSchema, TemporalIntegrity,
};

pub struct TemporalExampleProgram;
pub struct TemporalExampleFeature;

impl ApplicationFeature<TemporalHostSchema> for TemporalExampleFeature {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.query.example.temporal-feature.v1";
}

impl ApplicationProgramDefinition<TemporalHostSchema> for TemporalExampleProgram {
    type Contributions = (TemporalHostContribution,);
    type Actions = ApplicationActionList<
        ApplicationOperationActionRef<TemporalHostSchema, TemporalExampleFeature, AmendTemporal>,
        ApplicationActionList<
            ApplicationOperationActionRef<
                TemporalHostSchema,
                TemporalExampleFeature,
                AmendTemporalAndPublishDefinition,
            >,
            ApplicationActionList<
                ApplicationConditionalOperationActionRef<
                    TemporalHostSchema,
                    TemporalExampleFeature,
                    ExecuteTemporal,
                >,
                ApplicationActionList<
                    ApplicationOperationActionRef<
                        TemporalHostSchema,
                        TemporalExampleFeature,
                        RevokeTemporalPrincipal,
                    >,
                    ApplicationActionLeaf,
                >,
            >,
        >,
    >;
    type Features = ApplicationFeatureList<
        ApplicationFeatureRef<TemporalHostSchema, TemporalExampleFeature>,
        ApplicationFeatureLeaf,
    >;
    type OutputGraph = ApplicationNoOutputGraph;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationLocalRuleRef<TemporalHostSchema, TemporalExampleFeature, TemporalIntegrity>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.example.temporal-program.v1");
}

pub fn validated_program() -> ValidatedApplicationProgram<TemporalHostSchema, TemporalExampleProgram>
{
    ApplicationProgramAuthoring::<TemporalHostSchema, TemporalExampleProgram>::begin()
        .validated_program()
        .expect("the temporal example program must validate")
}
