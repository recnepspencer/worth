//! Active retained-branch loan custody, crash-consistent pending branch
//! retirement reconciliation, and installed branch lookup on
//! [`WorthQueryApplicationHome`].

use worth_execution::ExecutionAllocationPolicy;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryBootstrapAdvancementPhase,
        WorthQueryManagedComputationResourceDenial,
    },
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
    product::{
        WorthQueryApplicationProductBranchCloseDenial, WorthQueryProductBranch,
        WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchCloseDenial,
    },
};
use worth_relational::facade::config::ResidencyBudget;
use worth_relational::facade::identity::VersionId;
use worth_runtime_world::facade::HistoricalReadDenial;

use super::super::preview::WorthQueryPreviewFork;
use super::super::WorthQueryApplicationHome;
use super::custody_state::RetainedBranchCustody;
use super::query_stream::{
    corrupt, map_branch_stop, map_inventory_denial, map_work_ceiling_denial,
};
use crate::session::WorthQueryHostSession;

/// RAII fence keeping an active in-process `retain_historical_basis` branch protected.
pub(super) struct InflightRetainGuard<'home, Schema: ApplicationSchema, Program: Send + Sync> {
    home: &'home WorthQueryApplicationHome<Schema, Program>,
    branches: Vec<String>,
}

impl<'home, Schema: ApplicationSchema, Program: Send + Sync>
    InflightRetainGuard<'home, Schema, Program>
{
    pub(super) fn enter(
        home: &'home WorthQueryApplicationHome<Schema, Program>,
        branch_name: String,
    ) -> Self {
        home.custody_lock().begin_inflight_retain(&branch_name);
        Self {
            home,
            branches: vec![branch_name],
        }
    }

    pub(super) fn add_branch(&mut self, branch_name: String) {
        if !self.branches.contains(&branch_name) {
            self.home.custody_lock().begin_inflight_retain(&branch_name);
            self.branches.push(branch_name);
        }
    }
}

impl<Schema: ApplicationSchema, Program: Send + Sync> Drop
    for InflightRetainGuard<'_, Schema, Program>
{
    fn drop(&mut self) {
        let mut custody = self.home.custody_lock();
        for name in &self.branches {
            custody.finish_inflight_retain(name);
        }
    }
}

pub(super) struct QueryBranchLoanGuard<'home, Schema: ApplicationSchema, Program: Send + Sync> {
    pub(super) home: &'home WorthQueryApplicationHome<Schema, Program>,
    pub(super) branch_name: String,
    pub(super) budget: ResidencyBudget,
}

impl<Schema: ApplicationSchema, Program: Send + Sync> Drop
    for QueryBranchLoanGuard<'_, Schema, Program>
{
    fn drop(&mut self) {
        let branch = { self.home.custody_lock().release_loan(&self.branch_name) };
        let Some(branch) = branch else {
            return;
        };
        if self.home.close_branch(branch).is_ok() {
            self.home
                .mark_branch_retired_in_store(&self.branch_name, self.budget);
        } else {
            self.home
                .custody_lock()
                .restore_pending_prune(&self.branch_name, branch);
        }
    }
}

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: Send + Sync,
{
    pub(super) fn retire_pruned_branch_custody(
        &self,
        _session: &WorthQueryHostSession<'_, Schema, Program>,
        name: &str,
        budget: ResidencyBudget,
    ) -> Result<(), HistoricalReadDenial> {
        let resolved = self.resolve_installed_branch_by_name(name)?;
        let (branch_to_close, retired_absent) = {
            let mut custody = self.custody_lock();
            let branch = custody.request_prune(name, resolved);
            let absent = branch.is_none() && resolved.is_none() && !custody.has_active_loans(name);
            (branch, absent)
        };
        if let Some(branch) = branch_to_close {
            if self.close_branch(branch).is_ok() {
                self.mark_branch_retired_in_store(name, budget);
            } else {
                self.custody_lock().restore_pending_prune(name, branch);
            }
        } else if retired_absent {
            self.mark_branch_retired_in_store(name, budget);
        }
        Ok(())
    }

    pub(super) fn mark_branch_retired_in_store(&self, branch_name: &str, budget: ResidencyBudget) {
        if let Some(Ok(store)) = self.durable_historical_store_ref(budget) {
            let _ = store.mark_branch_retired(branch_name);
        } else if self.directory.is_none() {
            self.memory_historical_store_ref(self.memory_home_identity(), budget)
                .mark_branch_retired(branch_name);
        }
    }

    pub(super) fn record_inflight_retain_in_store(
        &self,
        branch_name: &str,
        budget: ResidencyBudget,
    ) -> Result<(), HistoricalReadDenial> {
        if let Some(store_res) = self.durable_historical_store_ref(budget) {
            store_res?.record_inflight_retain_intent(branch_name)
        } else {
            self.memory_historical_store_ref(self.memory_home_identity(), budget)
                .record_inflight_retain_intent(branch_name);
            Ok(())
        }
    }

    pub(super) fn abandon_inflight_retain_in_store(
        &self,
        branch_name: &str,
        budget: ResidencyBudget,
    ) {
        if let Some(Ok(store)) = self.durable_historical_store_ref(budget) {
            let _ = store.abandon_inflight_retain_intent(branch_name);
        } else if self.directory.is_none() {
            self.memory_historical_store_ref(self.memory_home_identity(), budget)
                .abandon_inflight_retain_intent(branch_name);
        }
    }

    pub(super) fn is_branch_protected_from_reconcile(
        &self,
        branch_name: &str,
        budget: ResidencyBudget,
    ) -> bool {
        let local_protected = {
            let custody = self.custody_lock();
            custody.has_active_loans(branch_name) || custody.is_inflight_retain(branch_name)
        };
        if local_protected {
            return true;
        }
        if let Some(Ok(store)) = self.durable_historical_store_ref(budget) {
            store.is_retained_branch(branch_name)
        } else if self.directory.is_none() {
            self.memory_historical_store_ref(self.memory_home_identity(), budget)
                .is_retained_branch(branch_name)
        } else {
            false
        }
    }

    pub(super) fn discard_unclaimed_preview(
        &self,
        preview: WorthQueryPreviewFork<'_, Schema>,
        branch_name: &str,
        budget: ResidencyBudget,
    ) {
        self.abandon_inflight_retain_in_store(branch_name, budget);
        match preview.discard(self) {
            Ok(_) => self.mark_branch_retired_in_store(branch_name, budget),
            Err(failure) => {
                let branch = failure.preview().branch();
                self.custody_lock()
                    .restore_pending_prune(branch_name, branch);
                drop(failure);
            }
        }
    }

    pub(super) fn reconcile_pending_retired_branches(
        &self,
        _session: &WorthQueryHostSession<'_, Schema, Program>,
        budget: ResidencyBudget,
    ) {
        let pending = if let Some(Ok(store)) = self.durable_historical_store_ref(budget) {
            store.pending_retire_branches()
        } else if self.directory.is_none() {
            self.memory_historical_store_ref(self.memory_home_identity(), budget)
                .pending_retire_branches()
        } else {
            return;
        };
        for branch_name in pending {
            if self.is_branch_protected_from_reconcile(&branch_name, budget) {
                continue;
            }
            if let Ok(resolved) = self.resolve_installed_branch_by_name(&branch_name) {
                if self.is_branch_protected_from_reconcile(&branch_name, budget) {
                    continue;
                }
                let confirmed_closed = match resolved {
                    Some(branch) => self.close_branch(branch).is_ok(),
                    None => true,
                };
                if confirmed_closed {
                    self.custody_lock().clear_retired(&branch_name);
                    self.mark_branch_retired_in_store(&branch_name, budget);
                }
            }
        }
    }

    pub(super) fn resolve_branch_snapshot_version(
        &self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        branch_name: Option<&str>,
    ) -> VersionId {
        let Some(name) = branch_name else {
            return VersionId(0);
        };
        let Ok(Some(branch)) = self.resolve_installed_branch_by_name(name) else {
            return VersionId(0);
        };
        session
            .for_branch(branch)
            .retain_read_observation()
            .map_or(VersionId(0), |obs| obs.relational_snapshot_version())
    }
}

impl<Schema: ApplicationSchema, Program> WorthQueryApplicationHome<Schema, Program> {
    pub(super) fn custody_lock(&self) -> std::sync::MutexGuard<'_, RetainedBranchCustody> {
        self.retained_branch_custody
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    pub(super) fn resolve_installed_branch_by_name(
        &self,
        target_name: &str,
    ) -> Result<Option<WorthQueryProductBranch>, HistoricalReadDenial> {
        let scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            WorthQueryAdmissionClock::system(),
        );
        self.primary_graph()
            .with_funded_application_completion(
                &scope,
                ExecutionAllocationPolicy::SystemAllocation,
                |phase| {
                    self.primary_graph()
                        .branches()
                        .inventory(&phase)
                        .map(|inv| inv.find_by_name(target_name))
                },
            )
            .map_err(|_| corrupt())?
            .map_err(|_| corrupt())
    }
}

/// Retires crash-leftover historical basis branches during Home open using an
/// explicit bootstrap-to-Home advancement handoff that preserves the opening
/// request's cancellation, deadline, work, and memory controls without spawning
/// a background thread.
pub(in crate::application_home) fn reconcile_persisted_historical_branches_on_open<
    Schema: ApplicationSchema,
>(
    bootstrap: &WorthQueryBootstrapAdvancementPhase<'_>,
    primary_graph: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    dir: &super::super::runtime::DirectoryHome,
) -> Result<RetainedBranchCustody, WorthQueryAdvancementDenial> {
    let Ok(pending) = dir.recover_persisted_historical_basis_catalog() else {
        return Ok(RetainedBranchCustody::default());
    };
    if pending.is_empty() {
        return Ok(RetainedBranchCustody::default());
    }
    let active_head = primary_graph.current_world();
    let (outcomes, stopped_during_reconcile) = primary_graph
        .with_bootstrap_handed_off_advancement(bootstrap, true, |phase| {
            let mut outcomes = Vec::with_capacity(pending.len());
            let mut stopped = None;
            for branch_name in &pending {
                if let Err(stop) = bootstrap.execution_request().consult() {
                    stopped = Some(map_work_ceiling_denial(stop));
                    break;
                }
                let inventory = match primary_graph.branches().inventory(&phase) {
                    Ok(inv) => inv,
                    Err(denial) => {
                        stopped = Some(map_inventory_denial(denial));
                        break;
                    }
                };
                let Some(branch) = inventory.find_by_name(branch_name) else {
                    outcomes.push((branch_name.clone(), None));
                    continue;
                };
                drop(inventory);
                if branch == active_head {
                    outcomes.push((branch_name.clone(), Some(branch)));
                    continue;
                }
                if let Err(stop) = bootstrap.execution_request().consult() {
                    stopped = Some(map_work_ceiling_denial(stop));
                    break;
                }
                match primary_graph.on_branch(branch).close(&phase) {
                    Ok(_)
                    | Err(WorthQueryApplicationProductBranchCloseDenial::Product(
                        WorthQueryProductBranchCloseDenial::AlreadyClosed,
                    )) => outcomes.push((branch_name.clone(), None)),
                    Err(WorthQueryApplicationProductBranchCloseDenial::ForeignPhase) => {
                        stopped = Some(WorthQueryAdvancementDenial::Resource(
                            WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase,
                        ));
                        break;
                    }
                    Err(WorthQueryApplicationProductBranchCloseDenial::Product(
                        WorthQueryProductBranchCloseDenial::Request(denial),
                    )) => {
                        stopped = Some(denial);
                        break;
                    }
                    Err(WorthQueryApplicationProductBranchCloseDenial::Product(
                        WorthQueryProductBranchCloseDenial::Selection(
                            WorthQueryProductBranchAdmissionDenial::Stopped(cause),
                        ),
                    )) => {
                        stopped = Some(map_branch_stop(cause));
                        break;
                    }
                    Err(_) => {
                        if let Err(stop) = bootstrap.execution_request().consult() {
                            stopped = Some(map_work_ceiling_denial(stop));
                            break;
                        }
                        outcomes.push((branch_name.clone(), Some(branch)));
                    }
                }
            }
            (outcomes, stopped)
        })?;
    let mut custody = RetainedBranchCustody::default();
    for (branch_name, unclosed) in outcomes {
        match unclosed {
            Some(branch) => custody.restore_pending_prune(&branch_name, branch),
            None => {
                let _ = dir.mark_persisted_historical_branch_retired(&branch_name);
            }
        }
    }
    if let Some(denial) = stopped_during_reconcile {
        return Err(denial);
    }
    Ok(custody)
}
