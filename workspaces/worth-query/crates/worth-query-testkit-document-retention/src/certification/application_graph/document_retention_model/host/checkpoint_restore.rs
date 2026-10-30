//! Restoring a P0-initial host from a checkpoint it captured, under whichever
//! roster the restoring host declares.

pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::application_installation::{
    in_memory_rostered_program_from_checkpoint, WorthQueryApplicationCheckpoint,
    WorthQueryApplicationProgramRoster, WorthQueryInMemoryApplicationDenial,
};
pub use worth_query_host::facade::declaration::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};

pub use super::super::{
    programs::{validated_first_program, RetentionProgramP0},
    schema::{DocumentRetentionContribution, DocumentRetentionSchema},
};
pub use super::{host_limits, DocumentRetentionRuntime};

pub fn restore_on_first_program(
    checkpoint: WorthQueryApplicationCheckpoint,
    roster: WorthQueryApplicationProgramRoster<'_, DocumentRetentionSchema>,
) -> Result<DocumentRetentionRuntime<RetentionProgramP0>, WorthQueryInMemoryApplicationDenial> {
    restore(validated_first_program(), checkpoint, roster)
}

pub fn restore<Initial>(
    initial: ValidatedApplicationProgram<DocumentRetentionSchema, Initial>,
    checkpoint: WorthQueryApplicationCheckpoint,
    roster: WorthQueryApplicationProgramRoster<'_, DocumentRetentionSchema>,
) -> Result<DocumentRetentionRuntime<Initial>, WorthQueryInMemoryApplicationDenial>
where
    Initial: ApplicationProgramDefinition<
            DocumentRetentionSchema,
            Contributions = (DocumentRetentionContribution,),
        > + 'static,
    Initial::Outputs:
        ApplicationProgramOutputsShape<DocumentRetentionSchema>
            + worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoots<
                DocumentRetentionSchema,
            >,
{
    in_memory_rostered_program_from_checkpoint(
        initial,
        roster.support(super::super::actor_generation::validated_actor_program()),
        DocumentRetentionSchema::declaration().expect("the document-retention schema is valid"),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        checkpoint,
    )
}
