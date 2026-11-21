//! Reconcile the assembled service without retaining its producer frames.
use super::*;
use crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalRuntimeInstallationDenialKind;
use worth_query_execution_query::facade::ports::ConditionalRuntimeInstallationDenialInternalPort as _;
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
        application,
    } = prepared.consume();
    conditional_operations.reconstruct_all(&application)?;
    let mut owner = application
        .inner
        .runtime
        .bridge
        .conditional_lifecycle()
        .map_err(|cause| {
            WorthQueryConditionalRuntimeInstallationDenial::from_kind_and_subject(
                WorthQueryConditionalRuntimeInstallationDenialKind::BridgeConditional(Box::new(
                    cause.kind(),
                )),
                cause.detail(),
            )
        })?;
    conditional_operations.reconcile_all(&mut owner)?;
    drop(owner);
    *application
        .conditional_cleanup
        .routes
        .operations
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = conditional_operations;
    Ok(application)
}
