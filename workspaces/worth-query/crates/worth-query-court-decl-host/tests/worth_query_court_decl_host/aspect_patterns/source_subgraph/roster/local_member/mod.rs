//! Keyed members and notes leave and return while their owner stays live.
use super::*;
mod operations;
pub use operations::*;
pub use operations::{Restoration, Restore, RestoreBinding, Restored};
mod correction;
mod effects;
pub use correction::UndoDelete;
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .application_mutation_binding::<DeleteBinding>()
        .declared_operation::<RestoreMember>(
            RestoreMember::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<RestoreBinding>()
}
pub fn configure(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    fault: world::Fault,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<DeleteBinding, _>(effects::Handler(fault))?;
    setup.handler::<RestoreBinding, _>(effects::Handler(fault))
}
