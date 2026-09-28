//! Restoring a P0-initial host from a checkpoint it captured, under whichever
//! roster the restoring host declares.

use worth_query_host::facade::application_installation::{
    in_memory_rostered_program_from_checkpoint, WorthQueryApplicationCheckpoint,
    WorthQueryApplicationProgramRoster, WorthQueryInMemoryApplicationDenial,
};

use super::super::{
    programs::{validated_first_program, RetentionProgramP0},
    schema::DocumentRetentionSchema,
};
use super::{host_limits, DocumentRetentionRuntime};

pub fn restore_on_first_program(
    checkpoint: WorthQueryApplicationCheckpoint,
    roster: WorthQueryApplicationProgramRoster<'_, DocumentRetentionSchema>,
) -> Result<DocumentRetentionRuntime<RetentionProgramP0>, WorthQueryInMemoryApplicationDenial> {
    in_memory_rostered_program_from_checkpoint(
        validated_first_program(),
        roster,
        DocumentRetentionSchema::declaration().expect("the document-retention schema is valid"),
        ((),),
        host_limits(),
        checkpoint,
    )
}
