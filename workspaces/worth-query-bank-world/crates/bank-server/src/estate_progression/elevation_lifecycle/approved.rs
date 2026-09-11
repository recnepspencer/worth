//! Bank-owned authority for the approved emergency-access phase.

use std::num::NonZeroUsize;

use bank_domain::schema::BankSchema;
use worth_query_host::facade::declaration::application_schema::ApplicationScalarValueBinding;
use worth_query_host::facade::primary_graph::{
    WorthQueryApprovedElevation, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQuerySelectedProductOperation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankEstateElevationRetentionWork {
    validated_intents_examined: usize,
    mutation_targets_materialized: usize,
    decision_facts_examined: usize,
    candidates_materialized: usize,
    demanded_loci_examined: usize,
}

impl BankEstateElevationRetentionWork {
    pub const fn validated_intents_examined(self) -> usize {
        self.validated_intents_examined
    }

    pub const fn mutation_targets_materialized(self) -> usize {
        self.mutation_targets_materialized
    }

    pub const fn decision_facts_examined(self) -> usize {
        self.decision_facts_examined
    }

    pub const fn candidates_materialized(self) -> usize {
        self.candidates_materialized
    }

    pub const fn demanded_loci_examined(self) -> usize {
        self.demanded_loci_examined
    }
}

/// Move-only Bank authority proving that the exact estate elevation was approved.
#[derive(Debug)]
pub struct BankApprovedEstateElevation {
    query: WorthQueryApprovedElevation,
}

impl BankApprovedEstateElevation {
    pub(in crate::estate_progression) const fn from_query(
        query: WorthQueryApprovedElevation,
    ) -> Self {
        Self { query }
    }

    pub(crate) const fn query(&self) -> &WorthQueryApprovedElevation {
        &self.query
    }

    pub(crate) fn into_query(self) -> WorthQueryApprovedElevation {
        self.query
    }

    pub(crate) fn select_approval_product<'runtime>(
        &self,
        application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
        maximum_history_entries: NonZeroUsize,
    ) -> Result<
        WorthQuerySelectedProductOperation<'runtime, BankSchema>,
        crate::BankApplicationQueryDenial,
    > {
        let publication = self.query.approval_product_publication();
        let history = application
            .branches()
            .history(application.current_world(), maximum_history_entries)
            .map_err(crate::BankApplicationQueryDenial::from_product_selection)?;
        let entry = history
            .entries()
            .find(|entry| entry.selected_commit() == publication.composite_commit())
            .ok_or(crate::BankApplicationQueryDenial::HistoricalCommitUnavailable)?;
        history
            .select(&entry)
            .map_err(crate::BankApplicationQueryDenial::from_product_selection)
    }

    pub fn requester_differs_from_approver(&self) -> bool {
        self.query.requester() != self.query.approver()
    }

    pub const fn approval_changed_record_count(&self) -> usize {
        self.query.approval_changed_record_count()
    }

    pub const fn approval_emitted_effect_count(&self) -> usize {
        self.query.approval_emitted_effect_count()
    }

    pub fn approval_retained_preimage_present(&self) -> bool {
        self.query.approval_retained_preimage().is_some()
    }

    pub fn request_retained_preimage_present(&self) -> bool {
        self.query.request_retained_preimage().is_some()
    }

    pub fn approval_prior_status_is_requested(&self) -> bool {
        use bank_domain::estate::EmergencyAccessStatus;
        use bank_domain::schema::EmergencyAccessStatusField;

        self.query
            .approval_retained_preimage()
            .and_then(|preimage| preimage.field_for(EmergencyAccessStatusField::reference()))
            .is_some_and(|field| {
                field.value()
                    == &bank_domain::schema::EmergencyAccessStatusBinding::encode(
                        &EmergencyAccessStatus::Requested,
                    )
                    .expect("declared emergency-access status must encode")
            })
    }

    pub fn approval_retention_work(&self) -> Option<BankEstateElevationRetentionWork> {
        self.query.approval_mutation_work().map(retention_work)
    }

    pub fn request_retention_work(&self) -> Option<BankEstateElevationRetentionWork> {
        self.query.request_mutation_work().map(retention_work)
    }
}

fn retention_work(
    work: &worth_query_host::facade::primary_graph::WorthQueryPrimaryMutationWorkEvidence,
) -> BankEstateElevationRetentionWork {
    BankEstateElevationRetentionWork {
        validated_intents_examined: work.preimage_validated_intents_examined(),
        mutation_targets_materialized: work.preimage_mutation_targets_materialized(),
        decision_facts_examined: work.preimage_decision_facts_examined(),
        candidates_materialized: work.preimage_candidates_materialized(),
        demanded_loci_examined: work.preimage_demanded_loci_examined(),
    }
}
