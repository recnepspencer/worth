//! The same pre-admitted terminal destination survives stop, refusal and unwind.
use super::*;
use crate::durability::{
    AdmittedWorldDisposition, PreparedWorldReleasedDisposition, WorldCommittedPublication,
    WorldDurableMediumDenial,
};
use crate::recovery::{PreparedReleasedOriginal, WorldReleasedOriginal};
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub(crate) enum RecoveredReleaseDestination {
    Prepared {
        disposition: Box<PreparedWorldReleasedDisposition>,
        destination: PreparedReleasedOriginal,
    },
    Acknowledged {
        disposition: AdmittedWorldDisposition,
        destination: PreparedReleasedOriginal,
    },
}
impl RecoveredReleaseDestination {
    fn prepare(
        request: &ExecutionRequest<'_, '_>,
        catalog: &super::super::catalog::ProductUnpublishedRecoveryCatalog,
        record: &RecoveredWorldPublicationRecord,
        subject: &WorldDurableArtifact,
    ) -> Result<Self, WorldDurableMediumDenial> {
        let destination = catalog.prepare_recovered_original_release(request, record)?;
        let disposition =
            PreparedWorldReleasedDisposition::prepare(request, &record.open, subject)?;
        Ok(Self::Prepared {
            disposition,
            destination,
        })
    }
    fn acknowledge(self, committed: &WorldCommittedPublication) -> Self {
        match self {
            Self::Prepared {
                disposition,
                destination,
            } => {
                let disposition = disposition.seal(committed);
                Self::Acknowledged {
                    disposition,
                    destination,
                }
            }
            acknowledged @ Self::Acknowledged { .. } => acknowledged,
        }
    }
    fn finish(self, native: Option<ReadmittedRelationalAssociatedEffect>) -> WorldReleasedOriginal {
        match self {
            Self::Acknowledged {
                disposition,
                destination,
            } => destination.finish_directory(disposition, native),
            Self::Prepared { .. } => {
                unreachable!("actual acknowledgement precedes Native transfer")
            }
        }
    }
}

pub(super) struct ReleaseDestinationLease {
    owner: Arc<RecoveredWorldPublicationRecord>,
    phase: Option<RecoveredReleaseDestination>,
}
impl ReleaseDestinationLease {
    pub(super) fn begin(
        request: &ExecutionRequest<'_, '_>,
        effects: &WorldRecoveredUnpublishedEffects,
        subject: &WorldDurableArtifact,
    ) -> Result<Self, WorldDurableMediumDenial> {
        let phase = effects
            .record
            .prepared_release
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        let phase = match phase {
            Some(phase) => phase,
            None => RecoveredReleaseDestination::prepare(
                request,
                &effects.catalog,
                &effects.record,
                subject,
            )?,
        };
        Ok(Self {
            owner: effects.record.clone(),
            phase: Some(phase),
        })
    }
    pub(super) fn acknowledge(&mut self, committed: &WorldCommittedPublication) {
        self.phase = Some(
            self.phase
                .take()
                .expect("same terminal destination")
                .acknowledge(committed),
        );
    }
    pub(super) fn finish(
        mut self,
        native: Option<ReadmittedRelationalAssociatedEffect>,
    ) -> WorldReleasedOriginal {
        self.phase
            .take()
            .expect("same terminal destination")
            .finish(native)
    }
}
impl Drop for ReleaseDestinationLease {
    fn drop(&mut self) {
        if let Some(phase) = self.phase.take() {
            *self
                .owner
                .prepared_release
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = Some(phase);
        }
    }
}
