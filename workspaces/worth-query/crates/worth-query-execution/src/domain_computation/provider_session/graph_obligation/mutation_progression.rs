use std::sync::Arc;

use crate::domain_computation::primary_graph::WorthQueryApplicationSnapshotRelease;
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledGraphObligationSetIdentity,
};
use worth_relational::facade::history::BranchId;
use worth_runtime_bridge::facade::TruthBranchIdentity;

use super::{
    WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity,
    WorthQueryManagedGraphWorkSession,
};

/// Private proof that the legacy managed-run engine is a worker of one exact
/// graph-work session rather than a second application-mutation authority.
pub(in crate::domain_computation) struct WorthQueryMutationRunBinding {
    session: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
    binding: ApplicationSchemaBindingIdentity,
    obligation: WorthQueryInstalledGraphObligationSetIdentity,
    relational_branch: BranchId,
    truth_branch: TruthBranchIdentity,
    worker: Arc<str>,
    provider_session: Arc<str>,
    resource_plan: Arc<str>,
    reservation_count: usize,
}

/// Cleanup owner after the exact live provider session has been associated
/// with the admitted mutation run.
pub(in crate::domain_computation) struct WorthQueryProviderSessionBoundMutationRun {
    run: WorthQueryMutationRunBinding,
    provider_session: super::super::WorthQueryProviderSessionTerminalBinding,
}

/// Non-authoritative terminal evidence for one exact mutation graph-work run.
///
/// Construction is private to successful managed-run cleanup. Consumers may
/// inspect this value, but no execution transition accepts it as authority.
#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryMutationGraphWorkCompletion {
    provider_session: Option<super::super::WorthQueryProviderSessionTerminalBinding>,
    session: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
    binding: ApplicationSchemaBindingIdentity,
    obligation: WorthQueryInstalledGraphObligationSetIdentity,
    relational_branch: BranchId,
    truth_branch: TruthBranchIdentity,
    snapshot_release: WorthQueryApplicationSnapshotRelease,
    attempt_resources_released: bool,
}

impl Eq for WorthQueryMutationGraphWorkCompletion {}

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn bind_mutation_run(
        &self,
        run: &crate::domain_computation::WorthQueryRunningDirectRun,
    ) -> Option<WorthQueryMutationRunBinding> {
        let affinity = run.graph_work_affinity()?;
        let identity = self.identity();
        let managed_run = self.managed_run_identity();
        let (provider_session, resource_plan, reservation_count) =
            run.mutation_resource_release_expectation();
        (affinity.session == identity && affinity.managed_run == managed_run).then(|| {
            WorthQueryMutationRunBinding {
                session: identity,
                managed_run,
                plan: self.plan_identity(),
                binding: self.binding().clone(),
                obligation: self.obligation().clone(),
                relational_branch: self.branch().relational().clone(),
                truth_branch: self.branch().truth().clone(),
                worker: run.identity().into(),
                provider_session: provider_session.into(),
                resource_plan: resource_plan.into(),
                reservation_count,
            }
        })
    }
}

impl WorthQueryMutationRunBinding {
    pub(in crate::domain_computation) fn bind_provider_session(
        self,
        provider_session: super::super::WorthQueryProviderSessionTerminalBinding,
    ) -> Result<WorthQueryProviderSessionBoundMutationRun, Self> {
        if provider_session.admits_mutation_run(self.session, self.managed_run, &self.worker) {
            Ok(WorthQueryProviderSessionBoundMutationRun {
                run: self,
                provider_session,
            })
        } else {
            Err(self)
        }
    }

    pub(in crate::domain_computation) fn finish(
        self,
        running: crate::domain_computation::WorthQueryRunningDirectRun,
        terminal: crate::domain_computation::WorthQueryManagedRunTerminalKind,
        snapshot_release: WorthQueryApplicationSnapshotRelease,
    ) -> Result<WorthQueryMutationGraphWorkCompletion, ()> {
        let affinity = running.graph_work_affinity().ok_or(())?;
        if running.identity() != self.worker.as_ref()
            || affinity.session != self.session
            || affinity.managed_run != self.managed_run
        {
            return Err(());
        }
        let cleanup = running
            .terminate_for_convergence(terminal)
            .cleanup()
            .map_err(|_| ())?;
        let inspection = cleanup.inspection();
        let attempt_resources_released = inspection.resources_released();
        if !(snapshot_release.custody_released()
            && inspection.run_identity() == self.worker.as_ref()
            && inspection.terminal() == terminal
            && inspection.provider_session_identity() == self.provider_session.as_ref()
            && attempt_resources_released
            && inspection.provider_work().provider_retained_bytes() == 0
            && inspection.resource_plan_identity() == self.resource_plan.as_ref()
            && inspection.released_reservation_count() == self.reservation_count)
        {
            return Err(());
        }
        Ok(WorthQueryMutationGraphWorkCompletion {
            provider_session: None,
            session: self.session,
            managed_run: self.managed_run,
            plan: self.plan,
            binding: self.binding,
            obligation: self.obligation,
            relational_branch: self.relational_branch,
            truth_branch: self.truth_branch,
            snapshot_release,
            attempt_resources_released,
        })
    }
}

impl WorthQueryProviderSessionBoundMutationRun {
    pub(in crate::domain_computation) fn finish(
        self,
        running: crate::domain_computation::WorthQueryRunningDirectRun,
        terminal: crate::domain_computation::WorthQueryManagedRunTerminalKind,
        snapshot_release: WorthQueryApplicationSnapshotRelease,
    ) -> Result<WorthQueryMutationGraphWorkCompletion, ()> {
        let mut completion = self.run.finish(running, terminal, snapshot_release)?;
        completion.provider_session = Some(self.provider_session);
        Ok(completion)
    }
}

impl WorthQueryMutationGraphWorkCompletion {
    pub(in crate::domain_computation) const fn provider_session_binding(
        &self,
    ) -> Option<&super::super::WorthQueryProviderSessionTerminalBinding> {
        self.provider_session.as_ref()
    }

    pub const fn session_identity(&self) -> WorthQueryGraphWorkSessionIdentity {
        self.session
    }

    pub const fn managed_run_identity(&self) -> WorthQueryGraphWorkManagedRunIdentity {
        self.managed_run
    }

    pub const fn plan_identity(&self) -> WorthQueryGraphWorkPlanIdentity {
        self.plan
    }

    pub const fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }

    pub const fn obligation_identity(&self) -> &WorthQueryInstalledGraphObligationSetIdentity {
        &self.obligation
    }

    pub const fn relational_branch(&self) -> &BranchId {
        &self.relational_branch
    }

    pub const fn truth_branch(&self) -> &TruthBranchIdentity {
        &self.truth_branch
    }

    pub const fn snapshot_released(&self) -> bool {
        self.snapshot_release.snapshot_released()
    }

    pub const fn snapshot_custody_released(&self) -> bool {
        self.snapshot_release.custody_released()
    }

    pub const fn attempt_resources_released(&self) -> bool {
        self.attempt_resources_released
    }
}
