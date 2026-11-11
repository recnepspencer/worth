//! Reconcile the assembled service without retaining its producer frames.
use super::*;
#[inline(never)]
pub(super) fn complete<S: ApplicationSchema + 'static>(
    prepared: BootstrapPublicationStorage<UnreconciledConditionalRuntime<S>>,
) -> Result<
    WorthQueryPrimaryGraphApplicationRuntime<S>,
    WorthQueryConditionalRuntimeInstallationDenial,
> {
    let UnreconciledConditionalRuntime {
        // Locals drop in reverse binding order: the application cleanup owner
        // closes Bridge resources while its dependent registry is still alive.
        mut conditional_operations,
        mut application,
    } = prepared.consume();
    conditional_operations.reconstruct_all(&application)?;
    conditional_operations
        .reconcile_all(&mut application.runtime.bridge.conditional_lifecycle())?;
    *application
        .conditional_cleanup
        .operations
        .get_mut()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = conditional_operations;
    Ok(application)
}
