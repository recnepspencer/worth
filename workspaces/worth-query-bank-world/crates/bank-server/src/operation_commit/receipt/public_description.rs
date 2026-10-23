//! Bank-owned scalar description copied at the execution/publication crossing.

use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;

use super::BankCommitCanonicalWorkPhases;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BankCommitPublicDescription {
    expected_version_count: usize,
    expected_fact_count: usize,
    canonical_work: BankCommitCanonicalWorkPhases,
    co_committed_dispatch_outbox: bool,
    retained_preimage: bool,
    performed_preimage_retention_work: bool,
}

impl BankCommitPublicDescription {
    pub(super) fn from_execution(execution: &WorthQueryApplicationCommitReceipt) -> Self {
        Self {
            expected_version_count: execution.precondition_comparison().expected_version_count(),
            expected_fact_count: execution.precondition_comparison().expected_fact_count(),
            canonical_work: BankCommitCanonicalWorkPhases::from_query(execution.canonical_work()),
            co_committed_dispatch_outbox: execution.dispatch_outbox().is_some(),
            retained_preimage: execution.has_retained_preimage(),
            performed_preimage_retention_work: execution.mutation_work().is_some_and(|work| {
                work.preimage_validated_intents_examined() != 0
                    || work.preimage_mutation_targets_materialized() != 0
                    || work.preimage_decision_facts_examined() != 0
                    || work.preimage_candidates_materialized() != 0
                    || work.preimage_demanded_loci_examined() != 0
            }),
        }
    }

    pub(super) const fn expected_version_count(&self) -> usize {
        self.expected_version_count
    }

    pub(super) const fn expected_fact_count(&self) -> usize {
        self.expected_fact_count
    }

    pub(super) const fn canonical_work(&self) -> BankCommitCanonicalWorkPhases {
        self.canonical_work
    }

    pub(super) const fn co_committed_dispatch_outbox(&self) -> bool {
        self.co_committed_dispatch_outbox
    }

    pub(super) const fn retained_preimage(&self) -> bool {
        self.retained_preimage
    }

    pub(super) const fn performed_preimage_retention_work(&self) -> bool {
        self.performed_preimage_retention_work
    }
}
