use worth_query_host::facade::declaration::application_program::{
    ApplicationCommitBoundary, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureSpec, ApplicationLocalRuleRef, ApplicationNoOutputGraph,
    ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramIdentity,
    ApplicationProgramOutputs, ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList,
    ValidatedApplicationProgram,
};

use super::application_entry::AmendTemporalBinding;
use super::schema::{
    ExecuteTemporal, RevokeTemporalPrincipal, TemporalHostContribution, TemporalHostSchema,
    TemporalIntegrity,
};

pub struct TemporalExampleProgram;
pub struct TemporalExampleFeature;

impl ApplicationFeature<TemporalHostSchema> for TemporalExampleFeature {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.query.example.temporal-feature.v1";
}

impl ApplicationProgramDefinition<TemporalHostSchema> for TemporalExampleProgram {
    type Contributions = (TemporalHostContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationLocalRuleRef<TemporalHostSchema, TemporalExampleFeature, TemporalIntegrity>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.example.temporal-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<TemporalHostSchema, TemporalExampleFeature>()
                .mutation::<AmendTemporalBinding>()
                .conditional_operation::<ExecuteTemporal>()
                .operation::<RevokeTemporalPrincipal>()
                .finish(),
        ]
    }
}

pub fn validated_program() -> ValidatedApplicationProgram<TemporalHostSchema, TemporalExampleProgram>
{
    ApplicationProgramAuthoring::<TemporalHostSchema, TemporalExampleProgram>::begin()
        .validated_program()
        .expect("the temporal example program must validate")
}
