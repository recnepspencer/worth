//! Demand uses the original backing and preserves sticky damage.
use super::*;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCauseOwnerPort as _;
impl RelationalCommitEnvelopeResidency {
    pub fn demand_positioned(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        self.demand_read(execution, limit)?
            .positioned
            .ok_or_else(|| {
                crate::durability::data::recovery_errors::DurabilityError::new(
                    crate::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                    "checkpoint envelope has no performed position",
                )
                .into()
            })
    }
    pub fn demand(
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
        crate::execution::request_custody::check_request(execution).map_err(Failure::Request)?;
        let journal = {
            let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
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
            match &state.backing {
                EnvelopeBacking::Memory { canonical, .. } => {
                    return Ok(ResidentEnvelopeRead {
                        envelope: Arc::clone(canonical.envelope()),
                        positioned: state
                            .position
                            .as_ref()
                            .and_then(RetainedCanonicalPosition::resident),
                    });
                }
                EnvelopeBacking::Journal { entry, source, .. } => {
                    Some((entry.position, Arc::clone(source)))
                }
                EnvelopeBacking::SelectedCheckpoint { .. } => None,
            }
        };
        let captured = match journal {
            Some((position, source)) => {
                match crate::durability::medium::RelationalDurableReadProtection::capture_journal_when_available(
                    &source, execution,
                ) {
                    Ok(protection) => Ok(Some((source, Some(protection)))),
                    Err(
                        crate::durability::medium::RelationalDurableOwnedReadRefusal::UnsupportedProvider,
                    ) => Ok(Some((source, None))),
                    Err(cause) => Err(
                        worth_relational_storage::internal_port::durability::medium::owned_read_stop::owned_stop(
                            cause,
                            RelationalStorageSubject::Envelope(position),
                        ),
                    ),
                }
            }
            None => Ok(None),
        };
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
        let position = match &state.backing {
            EnvelopeBacking::Journal { entry, .. }
            | EnvelopeBacking::SelectedCheckpoint { entry, .. } => entry.position,
            EnvelopeBacking::Memory { .. } => state.position.as_ref().map_or(
                crate::publication::patch::data::PatchStreamPosition(0),
                |position| position.position(),
            ),
        };
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
            EnvelopeBacking::Journal { entry, source, .. } => match &captured {
                Ok(Some((actual, Some(protection)))) if Arc::ptr_eq(actual, source) => {
                    entry.readmit_journal(source, Some(protection), execution, limit)
                }
                Ok(_) => entry.readmit_journal(source, None, execution, limit),
                Err(stop) => Err(stop.clone()),
            },
            EnvelopeBacking::SelectedCheckpoint { entry, directory } => {
                Self::readmit_selected(entry, directory.as_ref(), execution, limit)
            }
        }
        .map_err(|stop| map_demand_stop(stop, position));
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

fn map_demand_stop(
    stop: RelationalRecoveryStop,
    position: crate::publication::patch::data::PatchStreamPosition,
) -> Failure {
    match stop {
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
    }
}
