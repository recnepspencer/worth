//! A retained receipt and a readmitted original never become the same authority.
use super::super::{
    WorthQueryApplicationCommitReceipt, WorthQueryRecoveredApplicationCommitReceipt,
};
use super::WorthQueryOutputSourceReplay;
use worth_runtime_world::facade::{CompositeCommitIdentity, ProductBranchObservation};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputSourceEvidence {
    origin: SourceOrigin,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum SourceOrigin {
    Retained(WorthQueryApplicationCommitReceipt),
    Readmitted {
        original: WorthQueryRecoveredApplicationCommitReceipt,
        observation: ProductBranchObservation,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    },
}
impl From<WorthQueryApplicationCommitReceipt> for WorthQueryOutputSourceEvidence {
    fn from(receipt: WorthQueryApplicationCommitReceipt) -> Self {
        Self {
            origin: SourceOrigin::Retained(receipt),
        }
    }
}
impl WorthQueryOutputSourceEvidence {
    pub(in crate::domain_computation::primary_graph) fn readmitted(
        original: WorthQueryRecoveredApplicationCommitReceipt,
        observation: ProductBranchObservation,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    ) -> Self {
        Self {
            origin: SourceOrigin::Readmitted {
                original,
                observation,
                scope,
            },
        }
    }
    pub fn replay(&self) -> WorthQueryOutputSourceReplay<'_> {
        match &self.origin {
            SourceOrigin::Retained(receipt) => WorthQueryOutputSourceReplay::Retained(receipt),
            SourceOrigin::Readmitted { original, .. } => {
                WorthQueryOutputSourceReplay::Recorded(original)
            }
        }
    }
    pub fn retained_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        match &self.origin {
            SourceOrigin::Retained(receipt) => Some(receipt),
            SourceOrigin::Readmitted { .. } => None,
        }
    }
    pub fn recovered_receipt(&self) -> Option<&WorthQueryRecoveredApplicationCommitReceipt> {
        match &self.origin {
            SourceOrigin::Readmitted { original, .. } => Some(original),
            SourceOrigin::Retained(_) => None,
        }
    }
    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        match &self.origin {
            SourceOrigin::Retained(receipt) => receipt.product_branch(),
            SourceOrigin::Readmitted { observation, .. } => {
                crate::basis::WorthQueryProductBranch::from_occurrence(
                    observation.lifecycle_incarnation(),
                )
            }
        }
    }
    pub fn source_commit(&self) -> &CompositeCommitIdentity {
        match &self.origin {
            SourceOrigin::Retained(receipt) => {
                receipt.committed_product_publication().composite_commit()
            }
            SourceOrigin::Readmitted { observation, .. } => observation.selected_commit(),
        }
    }
    pub fn commit_reference(&self) -> &worth_relational::facade::history::RelationalCommitReceipt {
        match &self.origin {
            SourceOrigin::Retained(receipt) => receipt.commit_reference(),
            SourceOrigin::Readmitted { original, .. } => original.commit_reference(),
        }
    }
    pub fn is_selected_at(&self, observation: &ProductBranchObservation) -> bool {
        match &self.origin {
            SourceOrigin::Retained(receipt) => receipt
                .committed_product_publication()
                .is_selected_at(observation),
            SourceOrigin::Readmitted {
                observation: original,
                ..
            } => original == observation,
        }
    }
    pub fn matches_recorded_original(
        &self,
        original: &WorthQueryRecoveredApplicationCommitReceipt,
    ) -> bool {
        self.replay().matches_recorded(original)
    }
    pub fn same_retained_output_source_as(&self, other: &Self) -> bool {
        match (&self.origin, &other.origin) {
            (SourceOrigin::Retained(left), SourceOrigin::Retained(right)) => {
                left.same_retained_output_source_as(right)
            }
            (_, SourceOrigin::Readmitted { original, .. }) => {
                self.matches_recorded_original(original)
            }
            (SourceOrigin::Readmitted { original, .. }, SourceOrigin::Retained(right)) => {
                right.matches_recorded_original(original)
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn subject(&self) -> ([u8; 32], String) {
        match &self.origin {
            SourceOrigin::Retained(receipt) => (
                *receipt.idempotency_binding().key_identity(),
                receipt.idempotency_binding().intent_text(),
            ),
            SourceOrigin::Readmitted { original, .. } => (
                *original.idempotency_key_identity(),
                original.recorded_intent().to_owned(),
            ),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn scope(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding {
        match &self.origin {
            SourceOrigin::Retained(receipt) => receipt.principal_scope().scope(),
            SourceOrigin::Readmitted { scope, .. } => *scope,
        }
    }
}

impl From<&WorthQueryApplicationCommitReceipt> for WorthQueryOutputSourceEvidence {
    fn from(receipt: &WorthQueryApplicationCommitReceipt) -> Self {
        receipt.clone().into()
    }
}
impl From<&WorthQueryOutputSourceEvidence> for WorthQueryOutputSourceEvidence {
    fn from(evidence: &WorthQueryOutputSourceEvidence) -> Self {
        evidence.clone()
    }
}
