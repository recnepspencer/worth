use std::marker::PhantomData;

use worth_query_decl::facade::application_program::{
    ApplicationFeatureSpec, ApplicationNoOutputGraph, ApplicationProgramAuthoring,
    ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationRuleLeaf,
    ValidatedApplicationProgram,
};
use worth_query_decl::facade::application_schema::ApplicationSchemaComposition;

pub(super) struct DenialProgram<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: ApplicationSchemaComposition> ApplicationProgramDefinition<Schema>
    for DenialProgram<Schema>
{
    type Contributions = Schema::Contributions;
    type Outputs = worth_query_decl::facade::application_program::ApplicationProgramOutputs<
        ApplicationNoOutputGraph,
    >;
    type Rules = ApplicationRuleLeaf;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.denial-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        Vec::new()
    }
}

pub(super) fn validated_denial_program<Schema: ApplicationSchemaComposition>(
) -> ValidatedApplicationProgram<Schema, DenialProgram<Schema>> {
    ApplicationProgramAuthoring::<Schema, DenialProgram<Schema>>::begin()
        .validated_program()
        .expect("empty denial program has no feature or output graph obligations")
}
