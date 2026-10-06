//! Restoring a P0-initial host from a checkpoint it captured, under whichever
//! roster the restoring host declares.

use worth_query_host::facade::application_installation::{
    in_memory_rostered_program_from_checkpoint, WorthQueryApplicationCheckpoint,
    WorthQueryApplicationProgramRoster, WorthQueryInMemoryApplicationDenial,
};
use worth_query_host::facade::declaration::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};

use super::super::{
    programs::{validated_first_program, RetentionProgramP0},
    schema::{DocumentRetentionContribution, DocumentRetentionSchema},
};
use super::{host_limits, DocumentRetentionRuntime};

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
        roster,
        DocumentRetentionSchema::declaration().expect("the document-retention schema is valid"),
        ((),),
        host_limits(),
        checkpoint,
    )
}
