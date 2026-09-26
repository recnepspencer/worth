//! Restoring a P0-initial host from a checkpoint it captured, under whichever
//! roster the restoring host declares.

use worth_query_host::facade::application_installation::{
    in_memory_rostered_program_from_checkpoint, WorthQueryApplicationCheckpoint,
    WorthQueryApplicationProgramRoster, WorthQueryInMemoryApplicationDenial,
};

use super::super::{
    programs::{validated_first_program, DimensionProgramP0},
    schema::BoundedDimensionSchema,
};
use super::{host_limits, BoundedDimensionRuntime};

pub fn restore_on_first_program(
    checkpoint: WorthQueryApplicationCheckpoint,
    roster: WorthQueryApplicationProgramRoster<'_, BoundedDimensionSchema>,
) -> Result<BoundedDimensionRuntime<DimensionProgramP0>, WorthQueryInMemoryApplicationDenial> {
    in_memory_rostered_program_from_checkpoint(
        validated_first_program(),
        roster,
        BoundedDimensionSchema::declaration().expect("the bounded-dimension schema is valid"),
        ((),),
        host_limits(),
        checkpoint,
    )
}
