//! Demand uses the original backing and preserves sticky damage.
use super::*;
impl RelationalCommitEnvelopeResidency {
    pub(crate) fn demand_positioned(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        self.demand_read(execution, limit)?
            .positioned
            .ok_or_else(|| {
                crate::durability::data::DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    "checkpoint envelope has no performed position",
                )
                .into()
            })
    }
    pub(crate) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<CanonicalCommitEnvelope>, Failure> {
        self.demand_read(execution, limit).map(|read| read.envelope)
    }
    fn demand_read(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<ResidentEnvelopeRead, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(failure) = state.damage {
            return Err(failure.into());
        }
        if let Some(envelope) = state.resident_envelope() {
            let positioned = state
                .position
                .as_ref()
                .and_then(RetainedCanonicalPosition::resident);
            return Ok(ResidentEnvelopeRead {
                envelope,
                positioned,
            });
        }
        let position = state.position.as_ref().map_or(
            crate::publication::patch::data::PatchStreamPosition(0),
            |position| position.position(),
        );
        let result = match &state.backing {
            EnvelopeBacking::Memory { canonical, .. } => {
                return Ok(ResidentEnvelopeRead {
                    envelope: Arc::clone(canonical.envelope()),
                    positioned: state
                        .position
                        .as_ref()
                        .and_then(RetainedCanonicalPosition::resident),
                })
            }
            EnvelopeBacking::Journal { entry, source, .. } => {
                Self::readmit_positioned(entry, source.as_ref(), execution, limit)
            }
            EnvelopeBacking::SelectedCheckpoint { entry, directory } => {
                Self::readmit_selected(entry, directory, execution, limit)
            }
        }
        .map_err(|stop| match stop {
            RelationalRecoveryStop::Request(cause) => Failure::Request(cause),
            RelationalRecoveryStop::Content(cause) => Failure::Content(cause),
            RelationalRecoveryStop::Allocation(_) => RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(position),
                cause: Cause::MemoryExhausted,
            }
            .into(),
            RelationalRecoveryStop::AdmissionImage(_) => RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(position),
                cause: Cause::Damaged,
            }
            .into(),
            RelationalRecoveryStop::Durability(error) => error
                .storage_unavailable
                .unwrap_or(RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Envelope(position),
                    cause: Cause::Damaged,
                })
                .into(),
        });
        match result {
            Ok(commit) => {
                let digest = commit.authoritative_payload_digest().map_err(|_| {
                    let failure = RelationalStorageUnavailable {
                        subject: RelationalStorageSubject::Envelope(commit.position()),
                        cause: Cause::Damaged,
                    };
                    state.damage = Some(failure);
                    Failure::StorageUnavailable(failure)
                })?;
                state.payload_digest = Some(digest);
                let envelope = commit.canonical_arc().clone();
                state.position = Some(RetainedCanonicalPosition::from_positioned(&commit));
                state.lineage = Some(lineage_delta(commit.envelope()));
                Ok(ResidentEnvelopeRead {
                    envelope,
                    positioned: Some(commit),
                })
            }
            Err(stop) => {
                if let Failure::StorageUnavailable(failure) = &stop
                    && failure.cause.is_proven_damage()
                {
                    state.damage = Some(*failure);
                }
                Err(stop)
            }
        }
    }
}
