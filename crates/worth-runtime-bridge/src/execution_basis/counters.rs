#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BridgeExecutionBasisCounters {
    managed_intent_check_count: usize,
    signal_attempt_check_count: usize,
    signal_attempt_admission_count: usize,
    signal_queue_binding_count: usize,
    truth_basis_check_count: usize,
    reservation_check_count: usize,
    truth_materialization_count: usize,
}

impl BridgeExecutionBasisCounters {
    pub(crate) fn checked_managed_intent(&mut self) {
        self.managed_intent_check_count += 1;
    }

    pub(crate) fn checked_signal_attempt(&mut self) {
        self.signal_attempt_check_count += 1;
    }

    pub(crate) fn admitted_signal_attempt(&mut self) {
        self.signal_attempt_admission_count += 1;
    }

    pub(crate) fn bound_signal_queue(&mut self) {
        self.signal_queue_binding_count += 1;
    }

    pub(crate) fn checked_truth_basis(&mut self) {
        self.truth_basis_check_count += 1;
    }

    pub(crate) fn checked_reservation(&mut self) {
        self.reservation_check_count += 1;
    }

    pub(crate) fn materialized_truth(&mut self) {
        self.truth_materialization_count += 1;
    }

    pub fn managed_intent_check_count(&self) -> usize {
        self.managed_intent_check_count
    }

    pub fn signal_attempt_check_count(&self) -> usize {
        self.signal_attempt_check_count
    }

    pub fn signal_attempt_admission_count(&self) -> usize {
        self.signal_attempt_admission_count
    }

    pub fn signal_queue_binding_count(&self) -> usize {
        self.signal_queue_binding_count
    }

    pub fn truth_basis_check_count(&self) -> usize {
        self.truth_basis_check_count
    }

    pub fn reservation_check_count(&self) -> usize {
        self.reservation_check_count
    }

    pub fn truth_materialization_count(&self) -> usize {
        self.truth_materialization_count
    }
}
#[cfg(test)]
#[path = "counters_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
