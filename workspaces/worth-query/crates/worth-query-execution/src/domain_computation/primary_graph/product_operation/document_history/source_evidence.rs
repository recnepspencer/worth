//! Original data is compared under fresh history admission and actual selection.
use super::model::HistoryModelIdentity;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
    WorthQueryRecoveredApplicationCommitReceipt,
};
use worth_foundational::facade::RecordedProductBranchReference;
use worth_runtime_world::facade::ProductBranchObservation;

#[derive(Clone, Copy)]
pub(super) enum HistorySourceEvidence<'a> {
    Live(&'a WorthQueryApplicationCommitReceipt),
    Recorded(&'a WorthQueryRecoveredApplicationCommitReceipt),
}
impl HistorySourceEvidence<'_> {
    /// Reservation meaning has already been bound to the freshly admitted model,
    /// operation, schema, principal, preconditions and governed input.
    pub(super) fn matches(
        self,
        model: &HistoryModelIdentity,
        meaning: WorthQueryApplicationIdempotencyBinding,
    ) -> bool {
        match self {
            Self::Live(receipt) => {
                model.matches_receipt(receipt)
                    && receipt.idempotency_binding().canonical_key() == meaning.canonical_key()
                    && receipt.idempotency_binding().has_same_intent(meaning)
            }
            Self::Recorded(receipt) => {
                use crate::domain_computation::primary_graph::application_attempt::idempotency::WorthQueryRecordedIntentMatch as Intent;
                receipt.idempotency_key_identity() == &meaning.canonical_key()
                    && match meaning.match_recorded_intent(receipt.recorded_intent()) {
                        Ok(Intent::Same) => true,
                        Ok(Intent::Drift) | Ok(Intent::Unverifiable) | Err(_) => false,
                    }
            }
        }
    }
    pub(super) fn is_selected_at(self, actual: &ProductBranchObservation) -> bool {
        match self {
            Self::Live(receipt) => receipt
                .committed_product_publication()
                .is_selected_at(actual),
            Self::Recorded(receipt) => recorded_endpoint_matches(
                receipt.new_product_head(),
                &WorthQueryProductBranchReadIdentity::from_observation(actual),
            ),
        }
    }
    pub(super) fn follows(self, previous: &WorthQueryProductBranchReadIdentity) -> bool {
        match self {
            Self::Live(receipt) => receipt
                .committed_product_publication()
                .original_movement()
                .is_some_and(|movement| {
                    previous
                        == &WorthQueryProductBranchReadIdentity::from_reference_snapshot(
                            movement.old_product_head(),
                        )
                }),
            Self::Recorded(receipt) => {
                recorded_endpoint_matches(receipt.old_product_head(), previous)
            }
        }
    }
}
/// A recorded Native frame is never rewritten into a live allocation frame.
/// The authenticated immutable World endpoint identifies its actual component
/// basis; this comparison conveys no observation or publication authority.
fn recorded_endpoint_matches(
    recorded: &RecordedProductBranchReference,
    actual: &WorthQueryProductBranchReadIdentity,
) -> bool {
    let owner = actual.branch_identity().owner_identity();
    match recorded {
        RecordedProductBranchReference::Home(head) => {
            owner.durable() == Some(head.branch().owner())
                && actual.branch_identity().name().as_str() == head.branch().name()
                && actual.lifecycle_incarnation().ordinal() == head.incarnation().ordinal()
                && actual.reference_generation().get() == head.generation().get()
                && actual.selected_commit().ordinal() == head.selected_commit().ordinal()
        }
        RecordedProductBranchReference::Memory(head) => {
            owner.durable().is_none()
                && owner.get() == head.owner()
                && actual.branch_identity().name().as_str() == head.branch()
                && actual.lifecycle_incarnation().ordinal() == head.incarnation()
                && actual.reference_generation().get() == head.generation()
                && actual.selected_commit().ordinal() == head.selected_commit()
        }
    }
}
