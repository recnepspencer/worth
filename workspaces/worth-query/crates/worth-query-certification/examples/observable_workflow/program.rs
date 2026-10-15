//! The mixed program composes the existing owning actions with inbound archival.
use crate::document_retention_model::{
    programs::{RetentionProgramP0, WorkflowOwnerProgram},
    schema::{DocumentRetentionContribution, DocumentRetentionSchema},
};
use worth_query_decl::facade::application_program::*;

pub struct MixedProgram;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for MixedProgram {
    type Contributions = (
        DocumentRetentionContribution,
        crate::effect::ArchiveContribution,
    );
    type Outputs =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.examples.observable-workflow.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = WorkflowOwnerProgram::feature_specs();
        features.push(
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, crate::effect::ArchiveFeature>(
            )
            .operation_action::<crate::effect::ArchiveBinding, crate::effect::ArchiveDelivery>()
            .finish(),
        );
        features
    }
}

pub fn validated_program() -> ValidatedApplicationProgram<DocumentRetentionSchema, MixedProgram> {
    ApplicationProgramAuthoring::<DocumentRetentionSchema, MixedProgram>::begin()
        .validated_program()
        .expect("the mixed workflow program validates")
}
