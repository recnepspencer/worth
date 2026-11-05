//! Moved whole from `domain_computation::managed_run::counters`: the items below and every impl they own.

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryManagedRunCounters {
    query_runtime_check_count: usize,
    resource_attempt_check_count: usize,
    bridge_intent_check_count: usize,
    bridge_source_check_count: usize,
    relational_basis_check_count: usize,
    semantic_basis_check_count: usize,
}

impl WorthQueryManagedRunCounters {
    pub(crate) fn checked_query_runtime(&mut self) {
        self.query_runtime_check_count += 1;
    }

    pub(crate) fn checked_resource_attempt(&mut self) {
        self.resource_attempt_check_count += 1;
    }

    pub(crate) fn checked_bridge_intent(&mut self) {
        self.bridge_intent_check_count += 1;
    }

    pub(crate) fn checked_bridge_source(&mut self) {
        self.bridge_source_check_count += 1;
    }

    pub(crate) fn checked_relational_basis(&mut self) {
        self.relational_basis_check_count += 1;
    }

    pub(crate) fn checked_semantic_basis(&mut self) {
        self.semantic_basis_check_count += 1;
    }

    pub fn query_runtime_check_count(&self) -> usize {
        self.query_runtime_check_count
    }

    pub fn resource_attempt_check_count(&self) -> usize {
        self.resource_attempt_check_count
    }

    pub fn bridge_intent_check_count(&self) -> usize {
        self.bridge_intent_check_count
    }

    pub fn bridge_source_check_count(&self) -> usize {
        self.bridge_source_check_count
    }

    pub fn relational_basis_check_count(&self) -> usize {
        self.relational_basis_check_count
    }

    pub fn semantic_basis_check_count(&self) -> usize {
        self.semantic_basis_check_count
    }
}
