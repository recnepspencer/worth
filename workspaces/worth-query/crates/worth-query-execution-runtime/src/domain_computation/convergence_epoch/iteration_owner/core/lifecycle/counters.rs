use crate::domain_computation::convergence_epoch::WorthQueryConvergenceDomainWorkEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConvergenceEpochCounters {
    operation_authority_check_count: usize,
    contract_authority_check_count: usize,
    managed_run_authority_check_count: usize,
    graph_authority_check_count: usize,
    iteration_count: usize,
    provider_work_unit_count: u64,
    comparator_call_count: usize,
    progress_check_count: usize,
    repeated_state_probe_count: usize,
    incumbent_retention_count: usize,
    incumbent_replacement_count: usize,
    yield_count: usize,
    readmission_count: usize,
    cleanup_attempt_count: usize,
    cleanup_completion_count: usize,
}

impl WorthQueryConvergenceEpochCounters {
    pub(super) const fn empty() -> Self {
        Self {
            operation_authority_check_count: 0,
            contract_authority_check_count: 0,
            managed_run_authority_check_count: 0,
            graph_authority_check_count: 0,
            iteration_count: 0,
            provider_work_unit_count: 0,
            comparator_call_count: 0,
            progress_check_count: 0,
            repeated_state_probe_count: 0,
            incumbent_retention_count: 0,
            incumbent_replacement_count: 0,
            yield_count: 0,
            readmission_count: 0,
            cleanup_attempt_count: 0,
            cleanup_completion_count: 0,
        }
    }

    pub(super) fn checked_operation_authority(&mut self) {
        self.operation_authority_check_count += 1;
    }

    pub(super) fn checked_contract_authority(&mut self) {
        self.contract_authority_check_count += 1;
    }

    pub(super) fn checked_managed_run_authority(&mut self) {
        self.managed_run_authority_check_count += 1;
    }

    pub(super) fn checked_graph_authority(&mut self) {
        self.graph_authority_check_count += 1;
    }

    pub(super) fn began_iteration(&mut self) {
        self.iteration_count += 1;
    }

    pub(super) fn recorded_provider_work(&mut self, provider_work_units: u64) {
        self.provider_work_unit_count = self
            .provider_work_unit_count
            .saturating_add(provider_work_units);
    }

    pub(super) fn reconciled_provider_work_total(&mut self, provider_work_units: u64) {
        self.provider_work_unit_count = provider_work_units;
    }

    pub(super) fn recorded_domain_work(&mut self, work: &WorthQueryConvergenceDomainWorkEvidence) {
        self.comparator_call_count = self
            .comparator_call_count
            .saturating_add(work.comparator_call_count());
        self.progress_check_count = self
            .progress_check_count
            .saturating_add(work.progress_check_count());
        self.repeated_state_probe_count = self
            .repeated_state_probe_count
            .saturating_add(work.repeated_state_probe_count());
    }

    pub(super) fn retained_incumbent(&mut self) {
        self.incumbent_retention_count += 1;
    }

    pub(super) fn replaced_incumbent_set(&mut self) {
        self.incumbent_replacement_count += 1;
    }

    pub(super) fn yielded(&mut self) {
        self.yield_count += 1;
    }

    pub(super) fn readmitted(&mut self) {
        self.readmission_count += 1;
    }

    pub(super) fn attempted_cleanup(&mut self) {
        self.cleanup_attempt_count += 1;
    }

    pub(super) fn completed_cleanup(&mut self) {
        self.cleanup_completion_count += 1;
    }

    pub fn operation_authority_check_count(&self) -> usize {
        self.operation_authority_check_count
    }

    pub fn contract_authority_check_count(&self) -> usize {
        self.contract_authority_check_count
    }

    pub fn managed_run_authority_check_count(&self) -> usize {
        self.managed_run_authority_check_count
    }

    pub fn graph_authority_check_count(&self) -> usize {
        self.graph_authority_check_count
    }

    pub fn iteration_count(&self) -> usize {
        self.iteration_count
    }

    pub fn provider_work_unit_count(&self) -> u64 {
        self.provider_work_unit_count
    }

    pub fn comparator_call_count(&self) -> usize {
        self.comparator_call_count
    }

    pub fn progress_check_count(&self) -> usize {
        self.progress_check_count
    }

    pub fn repeated_state_probe_count(&self) -> usize {
        self.repeated_state_probe_count
    }

    pub fn incumbent_retention_count(&self) -> usize {
        self.incumbent_retention_count
    }

    pub fn incumbent_replacement_count(&self) -> usize {
        self.incumbent_replacement_count
    }

    pub fn yield_count(&self) -> usize {
        self.yield_count
    }

    pub fn readmission_count(&self) -> usize {
        self.readmission_count
    }

    pub fn cleanup_attempt_count(&self) -> usize {
        self.cleanup_attempt_count
    }

    pub fn cleanup_completion_count(&self) -> usize {
        self.cleanup_completion_count
    }
}
