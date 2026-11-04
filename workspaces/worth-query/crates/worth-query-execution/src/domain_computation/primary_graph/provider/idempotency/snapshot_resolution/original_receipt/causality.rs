//! Original causal row meaning, without a correction source or live authority.
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionSourceCoordinates,
};
use crate::domain_computation::primary_graph::provider::aftermath_causality::WorthQueryCanonicalCorrectionFact;
use worth_relational::facade::{history::RelationalCommitReceipt, transactions::RecordRef};

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryRecoveredAftermathCausality {
    fact: WorthQueryCanonicalCorrectionFact,
}

impl WorthQueryRecoveredAftermathCausality {
    pub(in crate::domain_computation::primary_graph::provider::idempotency::snapshot_resolution) fn from_source_fact(
        fact: WorthQueryCanonicalCorrectionFact,
    ) -> Self {
        Self { fact }
    }
    pub const fn role(&self) -> WorthQueryAftermathCausalRole {
        self.fact.role
    }
    pub fn parent(&self) -> &RelationalCommitReceipt {
        &self.fact.parent
    }
    pub fn child(&self) -> &RelationalCommitReceipt {
        &self.fact.child
    }
    pub fn fact_record(&self) -> &RecordRef {
        &self.fact.record
    }
    pub fn source(&self) -> &WorthQueryCorrectionSourceCoordinates {
        &self.fact.source
    }
    pub fn root(&self) -> &WorthQueryCorrectionSourceCoordinates {
        &self.fact.root
    }
    pub fn selected_alternative(&self) -> Option<&WorthQueryCorrectionSourceCoordinates> {
        self.fact.selected.as_ref()
    }
    pub fn active_reversal(&self) -> Option<&WorthQueryCorrectionSourceCoordinates> {
        self.fact.active.as_ref()
    }
    pub fn prior_transition(&self) -> Option<&WorthQueryCorrectionSourceCoordinates> {
        self.fact.prior.as_ref()
    }
}
