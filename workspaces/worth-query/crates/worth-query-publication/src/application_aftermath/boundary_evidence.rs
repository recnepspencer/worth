use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitPublicationSource, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationCommitTerminalKind, WorthQueryPrimaryMutationWorkEvidence,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use crate::domain_computation::WorthQueryPublishedApplicationCommitAttemptReleasePosture;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedApplicationCommitKind {
    Executed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPublishedCanonicalWork {
    basis_preparations: u32,
    digest_derivations: u32,
    canonical_entries: u32,
    canonical_encoded_bytes: usize,
    canonical_material_allocation_bytes: usize,
    sha256_input_bytes: usize,
    sha256_compression_blocks: usize,
    digest_text_materializations: u32,
}

impl WorthQueryPublishedCanonicalWork {
    pub(crate) const fn from_owner(work: WorthQueryCanonicalWorkEvidence) -> Self {
        Self {
            basis_preparations: work.basis_preparations(),
            digest_derivations: work.digest_derivations(),
            canonical_entries: work.canonical_entries(),
            canonical_encoded_bytes: work.canonical_encoded_bytes(),
            canonical_material_allocation_bytes: work.canonical_material_allocation_bytes(),
            sha256_input_bytes: work.sha256_input_bytes(),
            sha256_compression_blocks: work.sha256_compression_blocks(),
            digest_text_materializations: work.digest_text_materializations(),
        }
    }

    pub const fn basis_preparations(self) -> u32 {
        self.basis_preparations
    }

    pub const fn digest_derivations(self) -> u32 {
        self.digest_derivations
    }

    pub const fn canonical_entries(self) -> u32 {
        self.canonical_entries
    }

    pub const fn canonical_encoded_bytes(self) -> usize {
        self.canonical_encoded_bytes
    }

    pub const fn canonical_material_allocation_bytes(self) -> usize {
        self.canonical_material_allocation_bytes
    }

    pub const fn sha256_input_bytes(self) -> usize {
        self.sha256_input_bytes
    }

    pub const fn sha256_compression_blocks(self) -> usize {
        self.sha256_compression_blocks
    }

    pub const fn digest_text_materializations(self) -> u32 {
        self.digest_text_materializations
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPublishedMutationWork {
    decision_facts: usize,
    proposed_facts: usize,
    invariant_state_facts: usize,
    invariant_work_units: u64,
    relational_invariant_executions: usize,
    relational_invariant_results: usize,
    preimage_validated_intents_examined: usize,
    preimage_mutation_targets_materialized: usize,
    preimage_decision_facts_examined: usize,
    preimage_candidates_materialized: usize,
    preimage_demanded_loci_examined: usize,
    touched_record_count: usize,
}

impl WorthQueryPublishedMutationWork {
    fn from_owner(work: &WorthQueryPrimaryMutationWorkEvidence) -> Self {
        Self {
            decision_facts: work.decision_fact_count(),
            proposed_facts: work.proposed_fact_count(),
            invariant_state_facts: work.invariant_state_fact_count(),
            invariant_work_units: work.invariant_work_units(),
            relational_invariant_executions: work.relational_invariant_execution_count(),
            relational_invariant_results: work.relational_invariant_result_count(),
            preimage_validated_intents_examined: work.preimage_validated_intents_examined(),
            preimage_mutation_targets_materialized: work.preimage_mutation_targets_materialized(),
            preimage_decision_facts_examined: work.preimage_decision_facts_examined(),
            preimage_candidates_materialized: work.preimage_candidates_materialized(),
            preimage_demanded_loci_examined: work.preimage_demanded_loci_examined(),
            touched_record_count: work.touched_record_count(),
        }
    }

    pub const fn decision_fact_count(&self) -> usize {
        self.decision_facts
    }
    pub const fn proposed_fact_count(&self) -> usize {
        self.proposed_facts
    }
    pub const fn invariant_state_fact_count(&self) -> usize {
        self.invariant_state_facts
    }
    pub const fn invariant_work_units(&self) -> u64 {
        self.invariant_work_units
    }
    pub const fn relational_invariant_execution_count(&self) -> usize {
        self.relational_invariant_executions
    }
    pub const fn relational_invariant_result_count(&self) -> usize {
        self.relational_invariant_results
    }
    pub const fn preimage_validated_intents_examined(&self) -> usize {
        self.preimage_validated_intents_examined
    }
    pub const fn preimage_mutation_targets_materialized(&self) -> usize {
        self.preimage_mutation_targets_materialized
    }
    pub const fn preimage_decision_facts_examined(&self) -> usize {
        self.preimage_decision_facts_examined
    }
    pub const fn preimage_candidates_materialized(&self) -> usize {
        self.preimage_candidates_materialized
    }
    pub const fn preimage_demanded_loci_examined(&self) -> usize {
        self.preimage_demanded_loci_examined
    }
    pub const fn touched_record_count(&self) -> usize {
        self.touched_record_count
    }
}

/// Completed, portable description of one Query commit publication boundary.
/// It carries no runtime, branch, record, session, or causal identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPublishedApplicationCommitBoundaryEvidence {
    kind: WorthQueryPublishedApplicationCommitKind,
    mutation_work: Option<WorthQueryPublishedMutationWork>,
    changed_record_count: usize,
    emitted_effect_count: usize,
    publication_work: WorthQueryPublishedCanonicalWork,
    attempt_release: WorthQueryPublishedApplicationCommitAttemptReleasePosture,
}

impl WorthQueryPublishedApplicationCommitBoundaryEvidence {
    pub(crate) fn from_owner(receipt: &WorthQueryApplicationCommitReceipt) -> Self {
        let kind = match receipt.terminal().kind() {
            WorthQueryApplicationCommitTerminalKind::Executed => {
                WorthQueryPublishedApplicationCommitKind::Executed
            }
        };
        let attempt_release = publish_attempt_release(receipt);
        Self {
            kind,
            mutation_work: receipt
                .mutation_work()
                .map(WorthQueryPublishedMutationWork::from_owner),
            changed_record_count: receipt.changed_record_count(),
            emitted_effect_count: receipt.emitted_effect_count(),
            publication_work: WorthQueryPublishedCanonicalWork::from_owner(
                receipt.canonical_work().publication(),
            ),
            attempt_release,
        }
    }

    pub(crate) fn from_publication_source(
        source: &WorthQueryApplicationCommitPublicationSource,
    ) -> Self {
        let kind = match source.terminal_kind() {
            WorthQueryApplicationCommitTerminalKind::Executed => {
                WorthQueryPublishedApplicationCommitKind::Executed
            }
        };
        Self {
            kind,
            mutation_work: source
                .mutation_work()
                .map(WorthQueryPublishedMutationWork::from_owner),
            changed_record_count: source.changed_record_count(),
            emitted_effect_count: source.emitted_effect_count(),
            publication_work: WorthQueryPublishedCanonicalWork::from_owner(
                source.publication_work(),
            ),
            attempt_release: publish_attempt_release_posture(source.attempt_resources_released()),
        }
    }

    pub const fn kind(&self) -> WorthQueryPublishedApplicationCommitKind {
        self.kind
    }
    pub const fn mutation_work(&self) -> Option<&WorthQueryPublishedMutationWork> {
        self.mutation_work.as_ref()
    }
    pub const fn changed_record_count(&self) -> usize {
        self.changed_record_count
    }
    pub const fn emitted_effect_count(&self) -> usize {
        self.emitted_effect_count
    }
    pub const fn publication_work(&self) -> WorthQueryPublishedCanonicalWork {
        self.publication_work
    }
    pub const fn attempt_release(
        &self,
    ) -> WorthQueryPublishedApplicationCommitAttemptReleasePosture {
        self.attempt_release
    }
}

fn publish_attempt_release(
    receipt: &WorthQueryApplicationCommitReceipt,
) -> WorthQueryPublishedApplicationCommitAttemptReleasePosture {
    publish_attempt_release_posture(receipt.terminal().attempt_resources_released())
}

const fn publish_attempt_release_posture(
    attempt_resources_released: Option<bool>,
) -> WorthQueryPublishedApplicationCommitAttemptReleasePosture {
    match attempt_resources_released {
        None => WorthQueryPublishedApplicationCommitAttemptReleasePosture::NotAttempted,
        Some(true) => WorthQueryPublishedApplicationCommitAttemptReleasePosture::Released,
        Some(false) => WorthQueryPublishedApplicationCommitAttemptReleasePosture::ReleaseFailed,
    }
}

#[cfg(test)]
mod terminal_release_tests;
