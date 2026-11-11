//! One original charged destination survives the disposable Native call.
use super::{
    WorldDurableArtifact, WorldDurableMediumDenial, WorldOpenAttemptRecord, WorldRecord,
    WorldRelationalEvidence,
};
mod open_record_loan;
use open_record_loan::OpenRecordLoan;
mod byte_destination;
use byte_destination::OccurrenceByteDestination;
#[cfg(test)]
mod retention_tests;

#[derive(Debug)]
pub(crate) struct WorldOccurrenceCustody {
    origin: Option<super::PreparedCheckpointOccurrence>,
    open: Option<WorldOpenAttemptRecord>,
    destination: OccurrenceByteDestination,
    pub(crate) subject: Option<WorldDurableArtifact>,
    prepared_artifact: Option<super::PreparedWorldArtifact>,
}

impl WorldOccurrenceCustody {
    pub(crate) fn prepare(
        original: &WorldDurableArtifact,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorldDurableMediumDenial> {
        let open = original
            .open_attempt()
            .ok_or(WorldDurableMediumDenial::Frame(
                super::WorldDurableFrameDenial::IdentityMismatch,
            ))?;
        // Reserve the selected owner's entire envelope before either owner effect.
        let total = super::codec::occurrence_backing::admitted_length(
            original.as_bytes().len(),
            open.component_intent,
        )
        .map_err(WorldDurableMediumDenial::Frame)?;
        let (prepared_artifact, open) =
            super::PreparedWorldArtifact::prepare_occurrence(open, execution)?;
        let destination = OccurrenceByteDestination::prepare(total, execution)?;
        let origin = super::PreparedCheckpointOccurrence::prepare(execution)?;
        Ok(Self {
            origin: Some(origin),
            open: Some(open),
            destination,
            subject: None,
            prepared_artifact: Some(prepared_artifact),
        })
    }

    pub(crate) fn origin_needs_binding(&self) -> bool {
        self.origin
            .as_ref()
            .expect("attempt retains original slot")
            .needs_binding()
    }
    pub(crate) fn bind_origin(
        &mut self,
        basis: &crate::basis::AdmittedCompositeRuntimeWorldBasis,
        pins: crate::retention::HistoryRetentionObligation,
    ) {
        self.origin
            .as_mut()
            .expect("prepared origin before journal submission")
            .bind(basis, pins);
    }
    pub(crate) fn take_origin(&mut self) -> super::PreparedCheckpointOccurrence {
        self.origin
            .take()
            .expect("the same occurrence transfers its origin exactly once")
    }
    pub(crate) fn take_unretained_origin(&mut self) -> Option<super::PreparedCheckpointOccurrence> {
        self.origin.take()
    }

    pub(crate) fn finalize(
        &mut self,
        relational: WorldRelationalEvidence,
        publishing_incarnation: worth_foundational::facade::HomeIncarnation,
    ) -> Result<&WorldDurableArtifact, WorldDurableMediumDenial> {
        self.finalize_owners(
            super::artifact::WorldOccurrenceOwnerEvidence::RelationalOnly(relational),
            publishing_incarnation,
        )
    }
    pub(crate) fn finalize_owners(
        &mut self,
        owners: super::artifact::WorldOccurrenceOwnerEvidence,
        publishing_incarnation: worth_foundational::facade::HomeIncarnation,
    ) -> Result<&WorldDurableArtifact, WorldDurableMediumDenial> {
        if self.subject.is_none() {
            if self
                .open
                .as_ref()
                .expect("same pre-effect open")
                .component_intent
                != owners.component_intent()
            {
                return Err(super::WorldDurableFrameDenial::IdentityMismatch.into());
            }
            self.destination.backing()?;
            let record = OpenRecordLoan::take(&mut self.open, owners, publishing_incarnation);
            let backing = self.destination.backing()?;
            super::codec::occurrence_backing::fill(backing, record.record())?;
            backing
                .check_live()
                .map_err(WorldDurableMediumDenial::Allocation)?;
            let bytes = self.destination.seal()?;
            self.subject = Some(
                self.prepared_artifact
                    .take()
                    .expect("one pre-effect artifact allocation")
                    .complete_owned(record.commit(), bytes),
            );
        }
        Ok(self
            .subject
            .as_ref()
            .expect("same finalized original occurrence"))
    }
}
