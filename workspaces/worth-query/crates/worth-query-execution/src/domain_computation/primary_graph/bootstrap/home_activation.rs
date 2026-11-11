//! Bind the activation record identity; branch history owns revision admission.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphBootstrap<Schema> {
    pub(in crate::domain_computation::primary_graph) fn readmit_home_activation_record(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        root: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let published = self
            .program_activation
            .as_ref()
            .and_then(|cell| cell.published())
            .ok_or_else(|| home_activation_denial("initial Home activation was not published"))?;
        let execution = phase.execution_request();
        self.graph
            .integration_handle()
            .with_runtime(&execution, |native| {
                let branch = native
                    .read_truth()
                    .project_observation(&root.basis().relational_basis().observation())
                    .map_err(|cause| {
                        home_activation_denial(format!(
                            "Home root Native observation denied: {cause:?}"
                        ))
                    })?;
                let (actual, _) = program_activation_recovery::read_activation(
                    &execution,
                    &branch,
                    self.graph.layout.program_activation(),
                    usize::MAX,
                    "Home root activation record cannot be read",
                    |_| true,
                )?;
                if actual != published {
                    return Err(home_activation_denial(
                        "Home root names another activation record",
                    ));
                }
                Ok(())
            })
    }
}
fn home_activation_denial(detail: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    primary_graph_denial(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        detail,
    )
}
