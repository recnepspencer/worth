//! Readmit the rostered programme on each actual Home branch before Query service.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphBootstrap<Schema> {
    pub(in crate::domain_computation::primary_graph) fn readmit_home_branch_program(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        root: &worth_runtime_world::facade::ProductBranchObservation,
        roster: &worth_query_installation::facade::WorthQueryProgramSupportRoster<Schema>,
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
                    "Home branch does not activate a rostered programme",
                    |rendering| {
                        roster.entries().iter().any(|entry| {
                            super::super::program_occurrence::program_revision_rendering(
                                entry.revision(),
                            ) == *rendering
                        })
                    },
                )?;
                if actual != published {
                    return Err(home_activation_denial(
                        "Home branch names another activation record",
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
