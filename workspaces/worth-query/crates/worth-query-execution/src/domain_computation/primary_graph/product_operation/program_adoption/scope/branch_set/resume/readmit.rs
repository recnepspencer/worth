//! Rejoin stopped DATA to fresh World-issued branch selectors at Home service.
use super::*;
use crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn readmit_stopped_branch_set_adoptions(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<Vec<WorthQueryStoppedBranchSetAdoption>, WorthQueryBranchSetAdoptionResumeDenial>
    {
        use WorthQueryBranchSetAdoptionResumeDenial as Denial;
        let execution = phase
            .checked_execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| {
                Denial::Preparation(
                    WorthQueryBranchSetAdoptionPreparationDenial::ExecutionDenied(cause),
                )
            })?;
        let support = self.installed_program_support().ok_or(Denial::Obligation(
            QueryObligationDenial::SupportUnavailable,
        ))?;
        let observed = self
            .runtime
            .product_runtime
            .owner
            .observation_port()
            .observe_installed_product_branches(&execution)
            .map_err(Denial::Inventory)?;
        let branch = |name: &str, lifecycle: u64| {
            observed
                .branches()
                .iter()
                .find(|b| {
                    b.branch_identity().name().as_str() == name
                        && b.lifecycle_incarnation().ordinal() == lifecycle
                })
                .map(|b| WorthQueryProductBranch::from_occurrence(b.lifecycle_incarnation()))
                .ok_or(Denial::CoverageMismatch)
        };
        let mut result = Vec::new();
        for row in support.lifecycle().stopped_adoptions() {
            let target = support
                .lifecycle()
                .rostered_revision(row.target)
                .map_err(Denial::Obligation)?;
            let mut remaining = Vec::new();
            for (name, lifecycle, _) in &row.remaining {
                remaining.push(branch(name, *lifecycle)?);
            }
            let mut progress = Vec::new();
            for prior in row.performed {
                progress.push(WorthQueryBranchSetAdoptionProgress::RecordedPerformed {
                    branch: branch(&prior.branch, prior.lifecycle)?,
                    commit: prior.commit,
                    source: prior.source,
                    target: prior.target,
                });
            }
            progress.push(WorthQueryBranchSetAdoptionProgress::RecordedNoEffect {
                branch: remaining[0],
                diagnostic: row.stop_diagnostic,
            });
            result.push(WorthQueryStoppedBranchSetAdoption {
                remaining: remaining.into_boxed_slice(),
                progress,
                target,
                total_selection_work_units: row.selection_work as usize,
                owner: self.runtime.product_runtime.owner.owner_identity(),
                durable: Some(row.attempt),
                _support_custody: Vec::new(),
            });
        }
        Ok(result)
    }
}
