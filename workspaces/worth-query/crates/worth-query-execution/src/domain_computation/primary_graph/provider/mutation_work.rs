//! Mutation-work evidence for a committed application attempt (C2 / R8.1).
//!
//! Counters alone cannot derive an inverse. Touched-record identities are
//! required at construction and are derived only from the commit's
//! `changed_records` — never caller-supplied.

use worth_relational::facade::transactions::RecordRef;

/// One graph record touched by a committed mutation.
///
/// Identity is the Relational record reference produced by the commit. Query
/// does not invent a parallel record namespace.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryTouchedRecordIdentity {
    record: RecordRef,
}

impl WorthQueryTouchedRecordIdentity {
    pub(in crate::domain_computation::primary_graph) const fn from_commit_record(
        record: RecordRef,
    ) -> Self {
        Self { record }
    }

    pub const fn record(&self) -> &RecordRef {
        &self.record
    }
}

/// Invariant-phase counters retained until commit attaches touched records.
///
/// Not a public evidence type — C2 forbids constructing complete mutation work
/// without names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryPrimaryMutationWorkCounters {
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
    pub(super) const fn new(
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

    pub(in crate::domain_computation::primary_graph) const fn decision_fact_count(self) -> usize {
        self.decision_facts
    }

    pub(in crate::domain_computation::primary_graph) const fn proposed_fact_count(self) -> usize {
        self.proposed_facts
    }
}

/// Complete mutation-work evidence carried on a committed receipt (C2).
///
/// Constructible only by completing invariant counters with commit-derived
/// touched-record identities. No public constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryMutationWorkEvidence {
    decision_facts: usize,
    proposed_facts: usize,
    invariant_state_facts: usize,
    invariant_work_units: u64,
    relational_invariant_executions: usize,
    relational_invariant_results: usize,
    expected_step_key_lookups: usize,
    expected_step_duplicate_equalities: usize,
    preimage_validated_intents_examined: usize,
    preimage_mutation_targets_materialized: usize,
    preimage_decision_facts_examined: usize,
    preimage_candidates_materialized: usize,
    preimage_demanded_loci_examined: usize,
    performed_touch_validated_intents_examined: usize,
    performed_touch_targets_materialized: usize,
    performed_touch_owner_state_lookups: usize,
    performed_application_touches_admitted: usize,
    installed_touch_scopes_compared: usize,
    installed_read_touch_overlaps_observed: usize,
    touched_records: Vec<WorthQueryTouchedRecordIdentity>,
}

impl WorthQueryPrimaryMutationWorkEvidence {
    /// Complete mutation work from invariant counters and the commit's records.
    ///
    /// `changed_records` must be the exact slice Relational published on the
    /// commit that produced this work. Callers cannot invent identities.
    pub(in crate::domain_computation::primary_graph) fn from_commit_seal(
        seal: super::session_commit::WorthQueryMutationWorkCommitSeal,
    ) -> Self {
        let (counters, changed_records, preimage) = seal.into_parts();
        let touched_records = changed_records
            .into_iter()
            .map(WorthQueryTouchedRecordIdentity::from_commit_record)
            .collect();
        let touch_projection = counters.installed_touch_admission.projection_work();
        Self {
            decision_facts: counters.decision_facts,
            proposed_facts: counters.proposed_facts,
            invariant_state_facts: counters.invariant_state_facts,
            invariant_work_units: counters.invariant_work_units,
            relational_invariant_executions: counters.relational_invariant_executions,
            relational_invariant_results: counters.relational_invariant_results,
            expected_step_key_lookups: counters.expected_step_preparation_work.key_lookups(),
            expected_step_duplicate_equalities: counters
                .expected_step_preparation_work
                .duplicate_equalities(),
            preimage_validated_intents_examined: preimage.validated_intents_examined(),
            preimage_mutation_targets_materialized: preimage.mutation_targets_materialized(),
            preimage_decision_facts_examined: preimage.decision_facts_examined(),
            preimage_candidates_materialized: preimage.candidates_materialized(),
            preimage_demanded_loci_examined: preimage.demanded_loci_examined(),
            performed_touch_validated_intents_examined: touch_projection
                .validated_intents_examined(),
            performed_touch_targets_materialized: touch_projection.mutation_targets_materialized(),
            performed_touch_owner_state_lookups: touch_projection.owner_state_lookups(),
            performed_application_touches_admitted: counters
                .installed_touch_admission
                .validated_application_touches(),
            installed_touch_scopes_compared: counters
                .installed_touch_admission
                .installed_touch_scopes(),
            installed_read_touch_overlaps_observed: counters
                .installed_touch_admission
                .read_touch_overlaps(),
            touched_records,
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

    pub const fn expected_step_key_lookups(&self) -> usize {
        self.expected_step_key_lookups
    }

    pub const fn expected_step_duplicate_equalities(&self) -> usize {
        self.expected_step_duplicate_equalities
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

    pub const fn performed_touch_validated_intents_examined(&self) -> usize {
        self.performed_touch_validated_intents_examined
    }

    pub const fn performed_touch_targets_materialized(&self) -> usize {
        self.performed_touch_targets_materialized
    }

    pub const fn performed_touch_owner_state_lookups(&self) -> usize {
        self.performed_touch_owner_state_lookups
    }

    pub const fn performed_application_touches_admitted(&self) -> usize {
        self.performed_application_touches_admitted
    }

    pub const fn installed_touch_scopes_compared(&self) -> usize {
        self.installed_touch_scopes_compared
    }

    pub const fn installed_read_touch_overlaps_observed(&self) -> usize {
        self.installed_read_touch_overlaps_observed
    }

    /// Records this mutation touched, derived from the commit (C2).
    pub fn touched_records(&self) -> &[WorthQueryTouchedRecordIdentity] {
        &self.touched_records
    }

    pub fn touched_record_count(&self) -> usize {
        self.touched_records.len()
    }
}
