use super::*;
use crate::durability::{WorldDurableMediumDenial, WorldDurablePublicationStop, WorldRecord};

#[derive(Debug)]
pub enum WorldRecoveredReleaseOutcome {
    AdmissionRefused {
        cause: crate::recovery::RuntimeWorldRecoveryDenial,
        custody: WorldRecoveredUnpublishedEffects,
    },
    Released(WorldReleasedRecoveredEffects),
    Pending(WorldRecoveredUnpublishedEffects),
    Refused {
        cause: WorldDurableMediumDenial,
        custody: WorldRecoveredUnpublishedEffects,
    },
    EvidenceRequired(WorldRecoveredUnpublishedEffects),
    Indeterminate(WorldRecoveredUnpublishedEffects),
}

/// The original Native semantic evidence remains descriptive after disposition.
/// This is neither a live CommitResult nor a performed composite publication.
#[derive(Debug)]
pub struct WorldReleasedRecoveredEffects {
    identity: ProductUnpublishedOwnerEffectsIdentity,
    attempt: CompositePublicationAttemptIdentity,
    commit: CompositeCommitIdentity,
    effect: Option<ReadmittedRelationalAssociatedEffect>,
}
impl WorldReleasedRecoveredEffects {
    pub fn identity(&self) -> &ProductUnpublishedOwnerEffectsIdentity {
        &self.identity
    }
    pub fn attempt_identity(&self) -> &CompositePublicationAttemptIdentity {
        &self.attempt
    }
    pub fn reserved_commit_identity(&self) -> &CompositeCommitIdentity {
        &self.commit
    }
    pub fn into_cleanup(self) -> crate::recovery::ProductUnpublishedCleanup {
        // Recovered application opens retain Native custody, never forked component branches.
        // This is reachable only from this SAME original durable Released result.
        crate::recovery::ProductUnpublishedCleanup::new(Vec::new(), Vec::new())
    }
    pub fn native_effect(&self) -> Option<&ReadmittedRelationalAssociatedEffect> {
        self.effect.as_ref()
    }
}

impl WorldRecoveredUnpublishedEffects {
    /// Reached only through the current World's recovery service and admission.
    pub(crate) fn release_original(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorldRecoveredReleaseOutcome {
        if self.continuation_already_committed() {
            return WorldRecoveredReleaseOutcome::AdmissionRefused {
                cause: crate::recovery::RuntimeWorldRecoveryDenial::ContinuationAlreadyCommitted,
                custody: self,
            };
        }
        let preparing = self
            .record
            .continuation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some();
        if preparing {
            return WorldRecoveredReleaseOutcome::AdmissionRefused {
                cause: crate::recovery::RuntimeWorldRecoveryDenial::ContinuationAlreadyStarted,
                custody: self,
            };
        }
        let subject = {
            let existing = self
                .record
                .release
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            match existing {
                Some(subject) => subject,
                None => {
                    let relational = self
                        .native_effect()
                        .map(|effect| {
                            let actual = effect.subject();
                            crate::durability::WorldRelationalReleaseEvidence::from_original(
                                execution,
                                actual.receipt(),
                                actual.position().0,
                                *actual.artifact_digest(),
                            )
                        })
                        .transpose();
                    let relational = match relational {
                        Ok(relational) => relational,
                        Err(cause) => {
                            return WorldRecoveredReleaseOutcome::Refused {
                                cause,
                                custody: self,
                            }
                        }
                    };
                    let subject = match WorldDurableArtifact::from_record(
                        WorldRecord::Released {
                            association: self.record.open.association.clone(),
                            custody: self.record.open.custody,
                            relational,
                        },
                        execution,
                    ) {
                        Ok(subject) => subject,
                        Err(cause) => {
                            return WorldRecoveredReleaseOutcome::Refused {
                                cause,
                                custody: self,
                            }
                        }
                    };
                    *self
                        .record
                        .release
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) = Some(subject.clone());
                    subject
                }
            }
        };
        let already_committed = self
            .record
            .committed_release
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some();
        let publication = if already_committed {
            Ok(())
        } else {
            self.record
                .binding
                .publish(&subject, execution)
                .map(|committed| {
                    *self
                        .record
                        .committed_release
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) = Some(committed.into_witness());
                })
        };
        match publication {
            Ok(()) => {
                // End the immutable view before transferring the SAME Native proof.
                self.source = None;
                let native = std::mem::replace(
                    &mut *self.record.native.lock().unwrap_or_else(|e| e.into_inner()),
                    RecoveredNativeCustody::Released,
                );
                let effect = match native {
                    RecoveredNativeCustody::OpenOnly => None,
                    RecoveredNativeCustody::Relational(native) => {
                        let native = match Arc::try_unwrap(native) {
                            Ok(native) => native,
                            Err(native) => {
                                *self.record.native.lock().unwrap_or_else(|e| e.into_inner()) =
                                    RecoveredNativeCustody::Relational(native);
                                self.refresh_source();
                                return WorldRecoveredReleaseOutcome::Pending(self);
                            }
                        };
                        let (effect, before, before_lease, after, after_lease) =
                            native.consume(|native| native.into_retained_parts());
                        drop((before, before_lease, after, after_lease));
                        Some(effect)
                    }
                    RecoveredNativeCustody::Prepared(_) => {
                        unreachable!("continuation cannot release")
                    }
                    RecoveredNativeCustody::Released => unreachable!("released exactly once"),
                };
                self.catalog.remove_recovered(&self.record);
                WorldRecoveredReleaseOutcome::Released(WorldReleasedRecoveredEffects {
                    identity: self.record.identity.clone(),
                    attempt: self.record.attempt.clone(),
                    commit: self.record.commit.clone(),
                    effect,
                })
            }
            Err(WorldDurablePublicationStop::Pending | WorldDurablePublicationStop::Busy) => {
                WorldRecoveredReleaseOutcome::Pending(self)
            }
            Err(WorldDurablePublicationStop::Refused(cause)) => {
                WorldRecoveredReleaseOutcome::Refused {
                    cause,
                    custody: self,
                }
            }
            Err(WorldDurablePublicationStop::Unresolved) => {
                WorldRecoveredReleaseOutcome::Indeterminate(self)
            }
            Err(WorldDurablePublicationStop::EvidenceRequired) => {
                WorldRecoveredReleaseOutcome::EvidenceRequired(self)
            }
        }
    }
}
