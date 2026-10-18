//! Closed destruction custody for the actual installed program runtime.
use super::{ApplicationProgramDefinition, ApplicationSchema, WorthQueryProgramApplicationRuntime};

// No accessor or constructor: this trait only preserves the concrete runtime's
// destruction and structural auto-trait contracts behind the private field.
pub(super) trait PreviewRuntimeDestructionCustody: Send + Sync {}

impl<Schema, Program> PreviewRuntimeDestructionCustody
    for WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
}
