use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProgramOutputTraversalWork {
    discovery_queries: usize,
    discovery_rows: usize,
    gathered_demands: usize,
}

impl ProgramOutputTraversalWork {
    pub const fn discovered(rows: usize, demands: usize) -> Self {
        Self {
            discovery_queries: 1,
            discovery_rows: rows,
            gathered_demands: demands,
        }
    }

    pub fn include(&mut self, other: Self) {
        self.discovery_queries = self
            .discovery_queries
            .saturating_add(other.discovery_queries);
        self.discovery_rows = self.discovery_rows.saturating_add(other.discovery_rows);
        self.gathered_demands = self.gathered_demands.saturating_add(other.gathered_demands);
    }
}

/// Work spent settling program outputs: discovery queries and rows, gathered demands,
/// producer and delivery contacts, invariant facts, units and executions, and derived
/// publications. Checkpoint readmission work and its charged scratch bound are
/// carried from the admission owner separately from producer execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationProgramWork {
    discovery_queries: usize,
    discovery_rows: usize,
    gathered_demands: usize,
    producer_contacts: usize,
    delivery_contacts: usize,
    invariant_state_facts: usize,
    invariant_work_units: u64,
    invariant_executions: usize,
    derived_publications: usize,
    checkpoint_readmission_work_units: u64,
    checkpoint_readmission_work_bound: u64,
    checkpoint_readmission_peak_charged_preparation_bytes: u64,
}

impl WorthQueryApplicationProgramWork {
    pub(super) fn from_settlements<'settlement>(
        traversal: ProgramOutputTraversalWork,
        root: (
            Option<&'settlement WorthQueryApplicationCommitReceipt>,
            Option<&'settlement worth_query_execution::facade::primary_graph::WorthQueryOutputReadinessDeliveryEvidence>,
            u64,
            u64,
            u64,
        ),
        descendants: impl Iterator<
            Item = (
                Option<&'settlement WorthQueryApplicationCommitReceipt>,
                Option<&'settlement worth_query_execution::facade::primary_graph::WorthQueryOutputReadinessDeliveryEvidence>,
                u64,
                u64,
                u64,
            ),
        >,
    ) -> Self {
        let settlements = std::iter::once(root).chain(descendants);
        Self::from_all_settlements(traversal, settlements)
    }

    pub(super) fn from_all_settlements<'settlement>(
        traversal: ProgramOutputTraversalWork,
        settlements: impl Iterator<
            Item = (
                Option<&'settlement WorthQueryApplicationCommitReceipt>,
                Option<&'settlement worth_query_execution::facade::primary_graph::WorthQueryOutputReadinessDeliveryEvidence>,
                u64,
                u64,
                u64,
            ),
        >,
    ) -> Self {
        let mut producer_contacts = 0_usize;
        let mut delivery_contacts = 0_usize;
        let mut invariant_state_facts = 0_usize;
        let mut invariant_work_units = 0_u64;
        let mut invariant_executions = 0_usize;
        let mut derived_publications = 0_usize;
        let mut checkpoint_readmission_work_units = 0_u64;
        let mut checkpoint_readmission_work_bound = 0_u64;
        let mut checkpoint_readmission_peak_charged_preparation_bytes = 0_u64;
        for (receipt, readiness, readmission_work, readmission_bound, readmission_bytes) in
            settlements
        {
            checkpoint_readmission_work_units =
                checkpoint_readmission_work_units.saturating_add(readmission_work);
            checkpoint_readmission_work_bound =
                checkpoint_readmission_work_bound.saturating_add(readmission_bound);
            checkpoint_readmission_peak_charged_preparation_bytes =
                checkpoint_readmission_peak_charged_preparation_bytes.max(readmission_bytes);
            if let Some(readiness) = readiness {
                producer_contacts =
                    producer_contacts.saturating_add(readiness.producer_contact_count());
                delivery_contacts =
                    delivery_contacts.saturating_add(readiness.delivery_contact_count());
            }
            if let Some(receipt) = receipt {
                if let Some(work) = receipt.mutation_work() {
                    invariant_state_facts =
                        invariant_state_facts.saturating_add(work.invariant_state_fact_count());
                    invariant_work_units =
                        invariant_work_units.saturating_add(work.invariant_work_units());
                    invariant_executions = invariant_executions
                        .saturating_add(work.relational_invariant_execution_count());
                }
                derived_publications = derived_publications
                    .saturating_add(usize::from(receipt.changed_record_count() != 0));
            }
        }
        Self {
            discovery_queries: traversal.discovery_queries,
            discovery_rows: traversal.discovery_rows,
            gathered_demands: traversal.gathered_demands,
            producer_contacts,
            delivery_contacts,
            invariant_state_facts,
            invariant_work_units,
            invariant_executions,
            derived_publications,
            checkpoint_readmission_work_units,
            checkpoint_readmission_work_bound,
            checkpoint_readmission_peak_charged_preparation_bytes,
        }
    }

    pub const fn discovery_query_count(self) -> usize {
        self.discovery_queries
    }

    pub const fn discovery_row_count(self) -> usize {
        self.discovery_rows
    }

    pub const fn gathered_demand_count(self) -> usize {
        self.gathered_demands
    }

    pub const fn producer_contact_count(self) -> usize {
        self.producer_contacts
    }

    pub const fn delivery_contact_count(self) -> usize {
        self.delivery_contacts
    }

    pub const fn invariant_state_fact_count(self) -> usize {
        self.invariant_state_facts
    }

    pub const fn invariant_work_units(self) -> u64 {
        self.invariant_work_units
    }

    pub const fn invariant_execution_count(self) -> usize {
        self.invariant_executions
    }

    pub const fn derived_publication_count(self) -> usize {
        self.derived_publications
    }

    /// Sum of owner-charged checkpoint output comparisons for these demands.
    pub const fn checkpoint_readmission_work_units(self) -> u64 {
        self.checkpoint_readmission_work_units
    }

    /// Saturating sum of owner-computed charged-work ceilings for accepted
    /// checkpoint readmissions; excludes decoding, rejected candidates and
    /// adoption. `u64::MAX` marks saturation, not a finite aggregate guarantee.
    pub const fn checkpoint_readmission_work_bound(self) -> u64 {
        self.checkpoint_readmission_work_bound
    }

    /// Largest charged scratch bound for one checkpoint readmission. This is
    /// not measured allocation, retained output storage, or resident memory.
    pub const fn checkpoint_readmission_peak_charged_preparation_bytes(self) -> u64 {
        self.checkpoint_readmission_peak_charged_preparation_bytes
    }
}
