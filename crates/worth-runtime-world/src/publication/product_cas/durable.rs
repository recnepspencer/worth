//! Durable selection precedes completion of the same cell's live movement.
use super::CompositePublicationReady;
use crate::branch::ProductBranchReferenceCell;
use crate::durability::{
    WorldDurableArtifact, WorldDurableMediumBinding, WorldDurableMediumDenial,
    WorldDurablePublicationStop,
};
use crate::publication::{PerformedCompositePublication, RuntimeWorldCancellationToken};
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedOwnerEffects};
use std::sync::Arc;

#[derive(Debug)]
pub enum WorldDurablePublicationDenial {
    ConditionalDefinitionRequired,
    ConditionalDefinitionExpectedProduct,
    BridgeDefinitionOwnerRequired,
    BridgeDefinitionPreparation(
        Box<worth_runtime_bridge::facade::BridgeDurableDefinitionPublicationDenial>,
    ),
    OwnerUnavailable,
    ForeignOwner,
    StaleIncarnation,
    Native(worth_relational::facade::branch::RelationalRecordedComponentRefusal),
    OriginalNativeEvidenceRequired,
    OriginalSignalEvidenceRequired,
    Medium(WorldDurableMediumDenial),
    MovementAdmission,
    EvidenceRequired,
    Indeterminate,
    CancelledAfterEffect,
    DeadlineAfterEffect,
}

#[derive(Debug)]
#[must_use]
pub enum WorldDurablePublicationOutcome {
    Performed(PerformedCompositePublication),
    Pending(WorldPendingDurablePublication),
    ProductUnpublished(ProductUnpublishedOwnerEffects),
    RecoveryRequired {
        cause: WorldDurablePublicationDenial,
        custody: ProductUnpublishedOwnerEffects,
    },
}

/// The existing ready attempt and its original capacities remain linear.
/// Dropping the caller exposes the same owner record for recovery.
#[derive(Debug)]
pub struct WorldPendingDurablePublication {
    ready: CompositePublicationReady,
    binding: Arc<WorldDurableMediumBinding>,
    subject: WorldDurableArtifact,
    cell: ProductBranchReferenceCell,
    cancellation: RuntimeWorldCancellationToken,
}

impl CompositePublicationReady {
    pub(crate) fn original_relational_parts(
        &self,
    ) -> Option<(
        worth_relational::facade::history::RelationalCommitIdentity,
        worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        Arc<worth_relational::facade::transactions::CommitResult>,
    )> {
        self.owner_results.settled_relational_parts()
    }

    pub(crate) fn signal_durable_output(
        &self,
    ) -> Option<&worth_signal::facade::durability::SignalDurableAdvanceOutput> {
        self.owner_results.signal().durable_advance_output()
    }
    pub(crate) fn durable_component_intent(
        &self,
    ) -> Option<crate::durability::WorldOpenComponentIntent> {
        self.custody.durable_association()?;
        self.custody.durable_component_intent()
    }
    pub(crate) fn durable_recovery(
        self,
        cause: WorldDurablePublicationDenial,
    ) -> WorldDurablePublicationOutcome {
        WorldDurablePublicationOutcome::RecoveryRequired {
            cause,
            custody: self.custody.retain(
                ProductUnpublishedCause::WorldOccurrenceReadmissionRequired,
                None,
                crate::publication::RetainedCommitDisposition::KeepOriginal,
            ),
        }
    }

    pub(crate) fn begin_durable(
        mut self,
        cell: ProductBranchReferenceCell,
        evidence: crate::durability::WorldOccurrenceOwnerEvidence,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        cancellation: RuntimeWorldCancellationToken,
        clock: &crate::lifecycle::RuntimeWorldClock,
        control: &crate::lifecycle::RuntimeWorldOperationControlForTest,
    ) -> WorldDurablePublicationOutcome {
        let (binding, subject) = match self.custody.finalize_durable_occurrence(evidence) {
            Ok(prepared) => prepared,
            Err(cause) => {
                return self.durable_recovery(WorldDurablePublicationDenial::Medium(cause))
            }
        };
        if binding.occurrence_unresolved() {
            return self.durable_recovery(WorldDurablePublicationDenial::Indeterminate);
        }
        // Genuine Native evidence is retained; no product comparison, permit
        // or occurrence submission has happened. Polling never repeats this.
        control.before_product_compare_for_test();
        if self
            .custody
            .admit_durable_movement(&cell, &self.expected_head, subject.clone())
            .is_err()
        {
            return self.durable_recovery(WorldDurablePublicationDenial::MovementAdmission);
        }
        let cutoff =
            crate::publication::ProductMovementCutoff::new(&cancellation, clock, self.deadline);
        if let Err(cause) = cutoff.check() {
            self.custody.cancel_unsubmitted_movement();
            return self.durable_recovery(match cause {
                crate::publication::ProductMovementCutoffDenial::Cancelled => {
                    WorldDurablePublicationDenial::CancelledAfterEffect
                }
                crate::publication::ProductMovementCutoffDenial::Deadline => {
                    WorldDurablePublicationDenial::DeadlineAfterEffect
                }
            });
        }
        WorldPendingDurablePublication {
            ready: self,
            binding,
            subject,
            cell,
            cancellation,
        }
        .advance(execution)
    }
}

impl WorldPendingDurablePublication {
    pub fn unpublished_recovery_handle(&self) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.ready.custody.unpublished_recovery_handle()
    }

    pub fn retain(self) -> ProductUnpublishedOwnerEffects {
        self.ready.custody.retain(
            ProductUnpublishedCause::WorldOccurrencePending,
            None,
            crate::publication::RetainedCommitDisposition::KeepOriginal,
        )
    }

    pub fn advance(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorldDurablePublicationOutcome {
        match self
            .ready
            .custody
            .advance_durable_occurrence(&self.binding, &self.subject, execution)
        {
            Ok(()) => {
                let late = if self.cancellation.is_cancelled() {
                    crate::publication::CompositeLateCancellationPosture::RequestedAfterProductMovement
                } else {
                    crate::publication::CompositeLateCancellationPosture::NotRequested
                };
                // The actual witness is already retained by the authoritative
                // cell. Cancellation cannot undo selected durable movement.
                match super::movement::attempt_product_movement(
                    self.ready.into_inputs(),
                    &self.cell,
                    late,
                    None,
                ) {
                    crate::publication::RuntimeWorldPublicationOutcome::Performed(value) => {
                        WorldDurablePublicationOutcome::Performed(value)
                    }
                    crate::publication::RuntimeWorldPublicationOutcome::ProductUnpublished(
                        value,
                    ) => WorldDurablePublicationOutcome::ProductUnpublished(value),
                    crate::publication::RuntimeWorldPublicationOutcome::NoEffect(_) => {
                        unreachable!("selected durable movement")
                    }
                }
            }
            Err(WorldDurablePublicationStop::Pending | WorldDurablePublicationStop::Busy) => {
                WorldDurablePublicationOutcome::Pending(self)
            }
            Err(WorldDurablePublicationStop::Refused(cause)) => self
                .ready
                .durable_recovery(WorldDurablePublicationDenial::Medium(cause.into_cause())),
            Err(WorldDurablePublicationStop::Unresolved) => self
                .ready
                .durable_recovery(WorldDurablePublicationDenial::Indeterminate),
            Err(WorldDurablePublicationStop::EvidenceRequired) => self
                .ready
                .durable_recovery(WorldDurablePublicationDenial::EvidenceRequired),
        }
    }
}
