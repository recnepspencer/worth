//! One revision edits a live owner sparsely and moves its keyed members both ways.
use super::*;
mod correction;
mod effects;
mod mark;
mod operations;
pub use correction::RegisterRevision;
pub use mark::*;
pub use operations::*;
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema.application_mutation_binding::<ReviseBinding>()
}
pub fn configure(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<ReviseBinding, _>(effects::Handler)
}
