//! The transition target installs only Relational execution, as D16 requires.
use super::document;
use document::schema::DocumentRetentionSchema as Schema;
use worth_query_decl::facade::application_program::*;
pub(super) struct TargetProgram;
impl ApplicationProgramDefinition<Schema> for TargetProgram {
    type Contributions = (document::host::RelationalRetentionContribution,);
    type Outputs =
        <document::programs::RetentionProgramP1 as ApplicationProgramDefinition<Schema>>::Outputs;
    type Rules =
        <document::programs::RetentionProgramP1 as ApplicationProgramDefinition<Schema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("durable-workflow.relational-target.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        document::programs::RetentionProgramP1::feature_specs()
    }
}
