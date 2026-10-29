//! The currently activated source chooses permission, never the target or initial host program.
pub use super::*;
pub use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
pub use worth_query_host::facade::declaration::application_program::*;
pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::publish,
    programs::{validated_first_program, validated_second_program},
    schema::{DocumentRetentionContribution, DocumentRetentionSchema},
};
pub struct NoAdoptionProgram;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for NoAdoptionProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.no-program-adoption.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        RetentionProgramP0::feature_specs()
    }
}
pub fn no_adoption_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, NoAdoptionProgram> {
    ApplicationProgramAuthoring::begin()
        .validated_program()
        .unwrap()
}
