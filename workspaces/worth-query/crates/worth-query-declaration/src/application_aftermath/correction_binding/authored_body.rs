//! Associates an authored-body correction binding with its installed program action.
use super::{ApplicationCorrectionBinding, ApplicationInverseBinding};
use crate::application_program::{ApplicationAction, ApplicationProgramDefinition};
use crate::application_schema::ApplicationSchema;

/// Associates a `mode = authored_body` correction with its installed inverse
/// action in `Program`, so Host undo entry points prepare and run the exact
/// installed inverse without repeating the action marker type.
pub trait ApplicationAuthoredBodyCorrection<
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
>: ApplicationCorrectionBinding<Schema>
{
    type InverseAction: ApplicationAction<Schema, ApplicationInverseBinding<Schema, Self>>;
}

/// Associates a `mode = authored_body` correction with its installed source
/// action in `Program` for retained-meaning redo/reapplication.
pub trait ApplicationAuthoredBodyReapplication<
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
>: ApplicationAuthoredBodyCorrection<Schema, Program>
{
    type SourceAction: ApplicationAction<Schema, Self::Source>;
}
