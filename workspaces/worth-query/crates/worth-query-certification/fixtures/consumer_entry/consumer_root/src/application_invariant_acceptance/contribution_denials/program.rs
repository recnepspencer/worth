use std::marker::PhantomData;

use worth_query_decl::facade::application_program::{
    ApplicationCommitBoundary, ApplicationFeatureSpec, ApplicationLocalRuleRef,
    ApplicationNoOutputGraph, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList,
    ApplicationSharedRuleRef, ValidatedApplicationProgram,
};
use worth_query_decl::facade::application_schema::ApplicationSchemaComposition;
use worth_query_parameter_entry::{
    ParameterFeature, ParameterSchemaBinding, PositiveParameterCount,
};
use worth_query_topology_entry::{PositivePlanarTurn, TopologySchemaBinding};

pub(super) struct DenialProgram<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: ApplicationSchemaComposition + ParameterSchemaBinding + TopologySchemaBinding>
    ApplicationProgramDefinition<Schema> for DenialProgram<Schema>
{
    type Contributions = Schema::Contributions;
    type Outputs = worth_query_decl::facade::application_program::ApplicationProgramOutputs<
        ApplicationNoOutputGraph,
    >;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationLocalRuleRef<Schema, ParameterFeature, PositiveParameterCount>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleList<
            ApplicationRuleAt<
                ApplicationSharedRuleRef<Schema, PositivePlanarTurn>,
                ApplicationCommitBoundary,
            >,
            ApplicationRuleLeaf,
        >,
    >;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.denial-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, ParameterFeature>().finish()]
    }
}

pub(super) fn validated_denial_program<
    Schema: ApplicationSchemaComposition + ParameterSchemaBinding + TopologySchemaBinding,
>() -> ValidatedApplicationProgram<Schema, DenialProgram<Schema>> {
    ApplicationProgramAuthoring::<Schema, DenialProgram<Schema>>::begin()
        .validated_program()
        .expect("the denial program has one valid parameter feature")
}
