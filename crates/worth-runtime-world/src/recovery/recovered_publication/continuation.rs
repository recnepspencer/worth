//! Resume the SAME original occurrence after source-backed custody admission.
use super::*;
use crate::history::{ReadmittedPublicationRecord, ReservedCompositeCommitCapacity};
use crate::{
    branch::{ProductBranchHeadProtection, WorldProductMovementPermit},
    durability::*,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
pub enum WorldRecoveredContinuationDenial {
    Admission(crate::recovery::RuntimeWorldRecoveryDenial),
    Durable(WorldDurableMediumDenial),
    Readmission(Box<WorldReadmissionDenial>),
    History(crate::history::CompositeHistoryCatalogDenial),
    Retention(crate::retention::RetentionObligationDenial),
    PredecessorChanged,
    NoOwnerEffect,
}
#[derive(Debug)]
pub enum WorldRecoveredContinuationOutcome {
    Published {
        commit: CompositeCommitIdentity,
    },
    Pending(WorldRecoveredUnpublishedEffects),
    Refused {
        cause: WorldRecoveredContinuationDenial,
        custody: WorldRecoveredUnpublishedEffects,
    },
    EvidenceRequired(WorldRecoveredUnpublishedEffects),
    Indeterminate(WorldRecoveredUnpublishedEffects),
}
#[derive(Debug)]
pub(crate) enum OriginalContinuationDisposition {
    Undisposed(WorldDurableArtifact),
    Readmitted(AdmittedWorldDisposition),
    Committed(WorldCommittedPublication),
}
#[derive(Debug)]
pub(crate) struct PreparedRecoveredPublication {
    pub(crate) record: Arc<crate::history::PreparedReadmittedPublicationRecord>,
    pub(crate) sealed: Option<Arc<ReadmittedPublicationRecord>>,
    pub(crate) capacity: ReservedCompositeCommitCapacity,
    pub(crate) pins: Option<crate::retention::HistoryRetentionObligation>,
    pub(crate) transfer: Option<crate::retention::ProductHeadRetentionTransfer>,
    pub(crate) protection: Option<ProductBranchHeadProtection>,
    pub(crate) origin: Option<PreparedCheckpointOccurrence>,
    pub(crate) disposition: OriginalContinuationDisposition,
    pub(crate) occurrence: WorldDurableArtifact,
    pub(crate) committed: Option<WorldCommittedPublication>,
    pub(crate) permit: Option<WorldProductMovementPermit>,
    pub(crate) _funding: Option<ExecutionMemoryReservation>,
}
/// The catalog retains the same prepared phase even if a caller unwinds.
struct ContinuationLease {
    owner: Arc<RecoveredWorldPublicationRecord>,
    phase: Option<Box<PreparedRecoveredPublication>>,
}
impl Drop for ContinuationLease {
    fn drop(&mut self) {
        if let Some(phase) = self.phase.take() {
            *self
                .owner
                .continuation
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = Some(phase);
        }
    }
}
impl PreparedRecoveredPublication {
    pub(crate) fn disposition_committed(&self) -> bool {
        match self.disposition {
            OriginalContinuationDisposition::Undisposed(_) => false,
            OriginalContinuationDisposition::Readmitted(_)
            | OriginalContinuationDisposition::Committed(_) => true,
        }
    }
    pub(crate) fn into_home_retirement(self) -> crate::history::ReadmittedPublicationRetirement {
        let Self {
            record,
            sealed,
            capacity,
            pins,
            transfer,
            protection,
            origin,
            disposition,
            occurrence,
            committed,
            permit,
            _funding,
        } = self;
        // Close still borrows Native Serving. Remove these nonfinal local aliases
        // before the original source record cooperatively retires its exact leases.
        // No durable Released or product publication is asserted by this teardown.
        drop((
            sealed,
            capacity,
            pins,
            transfer,
            protection,
            origin,
            disposition,
            occurrence,
            committed,
            permit,
            _funding,
        ));
        record.into_retirement()
    }
    pub(crate) fn advance(
        &mut self,
        owner: &RecoveredWorldPublicationRecord,
        execution: &ExecutionRequest<'_, '_>,
        control: &crate::lifecycle::RuntimeWorldOperationControlForTest,
    ) -> Result<(), WorldDurablePublicationStop> {
        if let OriginalContinuationDisposition::Undisposed(subject) = &self.disposition {
            let committed = owner.binding.publish(subject, execution)?;
            self.disposition = OriginalContinuationDisposition::Committed(committed);
            control.recovered_continued_for_test();
        }
        // A source-admitted Continued is already durable. Never publish it twice.
        match &self.disposition {
            OriginalContinuationDisposition::Readmitted(source) => {
                debug_assert!(matches!(
                    source.disposition(),
                    RecoveredCustodyDisposition::Continued
                ));
            }
            OriginalContinuationDisposition::Committed(committed) => {
                let _ = committed.position();
            }
            OriginalContinuationDisposition::Undisposed(_) => {
                unreachable!("disposition committed above")
            }
        }
        if self.committed.is_none() {
            let committed = owner.binding.publish(&self.occurrence, execution)?;
            // Retain the actual outcome before any later work can fail.
            self.committed = Some(committed);
        }
        Ok(())
    }
}
impl WorldRecoveredUnpublishedEffects {
    pub(crate) fn advance_original(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
        control: &crate::lifecycle::RuntimeWorldOperationControlForTest,
    ) -> WorldRecoveredContinuationOutcome {
        use WorldRecoveredContinuationOutcome as Outcome;
        let phase = self
            .record
            .continuation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .expect("prepared original continuation");
        let mut lease = ContinuationLease {
            owner: self.record.clone(),
            phase: Some(phase),
        };
        let result =
            lease
                .phase
                .as_mut()
                .expect("original phase")
                .advance(&self.record, execution, control);
        if let Err(cause) = result {
            return match cause {
                WorldDurablePublicationStop::Pending | WorldDurablePublicationStop::Busy => {
                    Outcome::Pending(self)
                }
                WorldDurablePublicationStop::Refused(cause) => Outcome::Refused {
                    cause: WorldRecoveredContinuationDenial::Durable(cause),
                    custody: self,
                },
                WorldDurablePublicationStop::EvidenceRequired => Outcome::EvidenceRequired(self),
                WorldDurablePublicationStop::Unresolved => Outcome::Indeterminate(self),
            };
        }
        let phase = lease.phase.as_mut().expect("same original phase");
        if phase.protection.is_none() {
            let occurrence = match AdmittedWorldOccurrence::from_committed(
                &self.record.open,
                phase.committed.as_ref().expect("actual committed"),
            ) {
                Ok(occurrence) => occurrence,
                Err(cause) => {
                    return Outcome::Refused {
                        cause: WorldRecoveredContinuationDenial::Readmission(Box::new(cause)),
                        custody: self,
                    }
                }
            };
            let record = phase.record.seal(&occurrence);
            phase.sealed = Some(record.clone());
            let pins = phase.pins.take().expect("pre-admitted history pins");
            let head = phase
                .capacity
                .try_install_readmitted_product_head(
                    record.after.shared_commit(),
                    pins,
                    record.clone(),
                )
                .expect("pre-admitted original history capacity and exact pins");
            phase.protection = Some(
                ProductBranchHeadProtection::owner_issued(
                    record.after.clone(),
                    phase.transfer.take().expect("same funded transfer"),
                    head,
                )
                .expect("exact successor protection"),
            );
            let committed = phase.committed.take().expect("same actual occurrence");
            phase
                .permit
                .as_mut()
                .expect("same movement permit")
                .acknowledge(committed.into_witness());
            self.record
                .binding
                .retain_checkpoint_occurrence(phase.origin.take().expect("same prepared origin"));
        }
        let Some(permit) = phase.permit.take() else {
            return Outcome::Indeterminate(self);
        };
        if permit
            .publish_readmitted(
                &self.record.expected,
                &mut phase.protection,
                phase.sealed.as_ref().expect("actual sealed occurrence"),
            )
            .is_err()
        {
            return Outcome::Indeterminate(self);
        }
        // The original history is now selected, with original Native custody.
        self.catalog.remove_recovered(&self.record);
        self.source = None;
        let native = std::mem::replace(
            &mut *self.record.native.lock().unwrap_or_else(|e| e.into_inner()),
            RecoveredNativeCustody::Released,
        );
        drop(native);
        lease.phase = None;
        Outcome::Published {
            commit: self.record.commit.clone(),
        }
    }
}
