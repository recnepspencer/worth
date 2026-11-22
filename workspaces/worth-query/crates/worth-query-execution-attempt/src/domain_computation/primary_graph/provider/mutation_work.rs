//! Mutation-work evidence for a committed application attempt (C2 / R8.1).

pub use worth_query_execution_lineage::facade::primary_graph::provider::mutation_work::{
    WorthQueryPrimaryMutationWorkEvidence, WorthQueryTouchedRecordIdentity,
};

/// Invariant-phase counters retained until commit attaches touched records.
///
/// Not a public evidence type — C2 forbids constructing complete mutation work
/// without names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryMutationWorkCounters {
    decision_facts: usize,
    proposed_facts: usize,
    invariant_state_facts: usize,
    invariant_work_units: u64,
    relational_invariant_executions: usize,
    relational_invariant_results: usize,
    expected_step_preparation_work:
        super::super::application_attempt::WorthQueryExpectedEffectStepPreparationWork,
    installed_touch_admission:
        super::application_touch_admission::WorthQueryInstalledTouchAdmissionEvidence,
}

impl WorthQueryPrimaryMutationWorkCounters {
    pub const fn new(
        _mint: super::invariant_execution::WorthQueryInvariantWorkMint,
        decision_facts: usize,
        proposed_facts: usize,
        invariant_state_facts: usize,
        invariant_work_units: u64,
        relational_invariant_executions: usize,
        relational_invariant_results: usize,
        expected_step_preparation_work: super::super::application_attempt::WorthQueryExpectedEffectStepPreparationWork,
        installed_touch_admission: super::application_touch_admission::WorthQueryInstalledTouchAdmissionEvidence,
    ) -> Self {
        Self {
            decision_facts,
            proposed_facts,
            invariant_state_facts,
            invariant_work_units,
            relational_invariant_executions,
            relational_invariant_results,
            expected_step_preparation_work,
            installed_touch_admission,
        }
    }

    pub const fn decision_fact_count(self) -> usize {
        self.decision_facts
    }

    pub const fn proposed_fact_count(self) -> usize {
        self.proposed_facts
    }
}

/// Complete mutation work from invariant counters and the commit's records.
pub fn mutation_work_from_commit_seal(
    seal: super::session_commit::WorthQueryMutationWorkCommitSeal,
) -> WorthQueryPrimaryMutationWorkEvidence {
    let (counters, index_maintenance_work, touched_records, preimage) = seal.into_parts();
    let touch_projection = counters.installed_touch_admission.projection_work();
    WorthQueryPrimaryMutationWorkEvidence::from_parts(
        index_maintenance_work,
        counters.decision_facts,
        counters.proposed_facts,
        counters.invariant_state_facts,
        counters.invariant_work_units,
        counters.relational_invariant_executions,
        counters.relational_invariant_results,
        counters.expected_step_preparation_work.key_lookups(),
        counters
            .expected_step_preparation_work
            .duplicate_equalities(),
        preimage.validated_intents_examined(),
        preimage.mutation_targets_materialized(),
        preimage.decision_facts_examined(),
        preimage.candidates_materialized(),
        preimage.demanded_loci_examined(),
        touch_projection.validated_intents_examined(),
        touch_projection.mutation_targets_materialized(),
        touch_projection.owner_state_lookups(),
        counters
            .installed_touch_admission
            .validated_application_touches(),
        counters.installed_touch_admission.installed_touch_scopes(),
        counters.installed_touch_admission.read_touch_overlaps(),
        touched_records,
    )
}
