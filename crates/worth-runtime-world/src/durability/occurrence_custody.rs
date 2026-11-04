//! One original charged destination survives the disposable Native call.
use super::{
    WorldDurableArtifact, WorldDurableMediumDenial, WorldOpenAttemptRecord, WorldRecord,
    WorldRelationalEvidence,
};

#[derive(Debug)]
pub(crate) struct WorldOccurrenceCustody {
    origin: Option<super::PreparedCheckpointOccurrence>,
    open: WorldOpenAttemptRecord,
    backing: Option<worth_execution::ExecutionByteBuffer>,
    pub(crate) subject: Option<WorldDurableArtifact>,
}

impl WorldOccurrenceCustody {
    pub(crate) fn prepare(
        original: &WorldDurableArtifact,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorldDurableMediumDenial> {
        let WorldRecord::OpenAttempt(open) = &original.record else {
            unreachable!("the opened phase owns its original open artifact")
        };
        // Occurrence adds the actual Native receipt/position/digest and current
        // publishing incarnation to the identical open grammar (56 bytes).
        let total = super::codec::occurrence_backing::admitted_length(original.as_bytes().len())
            .map_err(WorldDurableMediumDenial::Frame)?;
        let backing =
            worth_execution::ExecutionByteBuffer::allocate(total, execution.allocation_policy())
                .map_err(WorldDurableMediumDenial::Allocation)?;
        let origin = super::PreparedCheckpointOccurrence::prepare(execution)?;
        Ok(Self {
            origin: Some(origin),
            open: open.clone(),
            backing: Some(backing),
            subject: None,
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

    pub(crate) fn finalize(
        &mut self,
        relational: WorldRelationalEvidence,
        publishing_incarnation: worth_foundational::facade::HomeIncarnation,
    ) -> Result<&WorldDurableArtifact, WorldDurableMediumDenial> {
        if self.subject.is_none() {
            let record = WorldRecord::Occurrence {
                open: self.open.clone(),
                relational,
                publishing_incarnation,
            };
            let backing = self
                .backing
                .as_mut()
                .ok_or(WorldDurableMediumDenial::Unavailable)?;
            super::codec::occurrence_backing::fill(backing, &record)?;
            backing
                .check_live()
                .map_err(WorldDurableMediumDenial::Allocation)?;
            let bytes = self
                .backing
                .take()
                .expect("same filled backing")
                .seal()
                .map_err(WorldDurableMediumDenial::Allocation)?;
            self.subject = Some(WorldDurableArtifact::from_preallocated(record, bytes));
        }
        Ok(self
            .subject
            .as_ref()
            .expect("same finalized original occurrence"))
    }
}
