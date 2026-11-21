//! Runtime restoration of an interim checkpoint registry on an installed application.

use worth_execution::ExecutionRequest;
use worth_query_execution_lineage::facade::primary_graph::application_checkpoint::interim_transport::{
    application_binding, native_refusal, world_refusal, WorthQueryInterimCheckpointOrigin,
};

use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};

impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub(in crate::domain_computation::primary_graph) fn restore_interim_registry(
        &self,
        origin: &WorthQueryInterimCheckpointOrigin,
        continuation: Option<
            &worth_relational::facade::durability::NativeInterimBootstrapContinuation,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let receiver = self
            .runtime
            .primary_provider
            .graph
            .with_runtime(request, |runtime| {
                runtime
                    .durability_authority()
                    .admit_interim_receiver(&origin.native, request)
            })
            .map_err(|cause| native_refusal(cause).into_installation_denial())?;
        let product = &self.runtime.product_runtime;
        let lease = product
            .admit_product_occurrence(product.default_occurrence)
            .map_err(|cause| {
                Denial::new(
                    Kind::InterimCheckpointReceiverMismatch,
                    format!("interim World receiver refused: {cause:?}"),
                )
            })?;
        let port = product.interim_checkpoint_port().ok_or_else(|| {
            Denial::new(
                Kind::InterimCheckpointReceiverMismatch,
                "interim checkpoint cannot restore a directory Home",
            )
        })?;
        port.restore(
            request,
            lease.observation(),
            &receiver,
            &origin.native,
            &origin.world,
            continuation,
            // The continuation joins the authenticated source to the actual
            // acknowledged successor. Ordinary restore still requires the
            // current publication's exact application identity.
            if continuation.is_some() {
                origin.source_application
            } else {
                application_binding(self.publication())
            },
        )
        .map_err(|cause| world_refusal(cause).into_installation_denial())
    }
}
