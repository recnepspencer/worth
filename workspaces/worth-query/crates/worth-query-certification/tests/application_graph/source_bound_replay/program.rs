//! Actual ordinary action ownership for the source-bound replay proof.
use crate::document_retention_model::{
    assessment_output::{OrdinaryRetentionAssessmentBinding, PublishRetentionAssessment},
    programs::{DocumentRetentionFeature, RetentionProgramP0},
    retention_entry::SetRetentionBinding,
    schema::DocumentRetentionSchema,
};
use worth_query_host::facade::declaration::application_program::{
    ApplicationFeatureSpec, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ValidatedApplicationProgram,
};

pub(super) struct SourceBoundReplayProgram;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for SourceBoundReplayProgram {
    type Contributions = <RetentionProgramP0 as ApplicationProgramDefinition<
        DocumentRetentionSchema,
    >>::Contributions;
    type Outputs =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.source-bound-replay.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        // This is an ordinary invocation of the installed assessment handler.
        // The separate ordinary operation leaves the installed conditional supplier intact.
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
                .mutation::<SetRetentionBinding>()
                .mutation::<OrdinaryRetentionAssessmentBinding>()
                .conditional_operation::<PublishRetentionAssessment>()
                .finish(),
        ]
    }
}

pub(super) fn validated_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, SourceBoundReplayProgram> {
    ApplicationProgramAuthoring::<DocumentRetentionSchema, SourceBoundReplayProgram>::begin()
        .validated_program()
        .expect("the real source-bound action program must validate")
}
