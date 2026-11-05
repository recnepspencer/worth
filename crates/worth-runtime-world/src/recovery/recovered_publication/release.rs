use super::*;
use crate::durability::{
    WorldArtifactInput, WorldDurableMediumDenial, WorldDurablePublicationStop,
};

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
    pub fn native_effect(&self) -> Option<&ReadmittedRelationalAssociatedEffect> {
        self.effect.as_ref()
    }
}

impl WorldRecoveredUnpublishedEffects {
    /// Reached only through the current World's recovery service and admission.
    pub(crate) fn release_original(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorldRecoveredReleaseOutcome {
        let continuing = {
            let native = self
                .record
                .native
                .lock()
                .unwrap_or_else(|cause| cause.into_inner());
            match &*native {
                RecoveredNativeCustody::Continuing(_) | RecoveredNativeCustody::Working => {
                    Some(crate::recovery::RuntimeWorldRecoveryDenial::Busy)
                }
                RecoveredNativeCustody::Relational(native)
                    if native
                        .custody()
                        .effect()
                        .subject()
                        .association()
                        .is_some_and(|association| {
                            worth_foundational::facade::HomeProgramTransitionCaller::recognizes(
                                association.as_bytes(),
                            )
                        }) =>
                {
                    Some(crate::recovery::RuntimeWorldRecoveryDenial::HomeProgramTransitionRequired)
                }
                RecoveredNativeCustody::OpenOnly
                | RecoveredNativeCustody::Relational(_)
                | RecoveredNativeCustody::Released => None,
            }
        };
        if let Some(cause) = continuing {
            return WorldRecoveredReleaseOutcome::AdmissionRefused {
                cause,
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
                    let relational =
                        match &*self.record.native.lock().unwrap_or_else(|e| e.into_inner()) {
                            RecoveredNativeCustody::OpenOnly => Ok(None),
                            RecoveredNativeCustody::Relational(native) => {
                                let actual = native.custody().effect().subject();
                                crate::durability::WorldRelationalReleaseEvidence::from_original(
                                    execution,
                                    actual.receipt(),
                                    actual.position().0,
                                    *actual.artifact_digest(),
                                )
                                .map(Some)
                            }
                            RecoveredNativeCustody::Released => {
                                unreachable!("same unreleased claim")
                            }
                            RecoveredNativeCustody::Continuing(_)
                            | RecoveredNativeCustody::Working => unreachable!(
                                "the consuming claim refused continuing custody before release"
                            ),
                        };
                    let relational = match relational {
                        Ok(relational) => relational,
                        Err(cause) => {
                            return WorldRecoveredReleaseOutcome::Refused {
                                cause,
                                custody: self,
                            }
                        }
                    };
                    let subject = match WorldDurableArtifact::from_input(
                        WorldArtifactInput::Released {
                            association: &self.record.open.association,
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
        match self.record.binding.publish(&subject, execution) {
            Ok(committed) => {
                // Preserve the exact sealed witness before catalog mutation or
                // dropping any genuine Native predecessor/successor lease.
                *self
                    .record
                    .committed_release
                    .lock()
                    .unwrap_or_else(|e| e.into_inner()) = Some(committed);
                self.catalog.remove_recovered(&self.record);
                let native = std::mem::replace(
                    &mut *self.record.native.lock().unwrap_or_else(|e| e.into_inner()),
                    RecoveredNativeCustody::Released,
                );
                let effect = match native {
                    RecoveredNativeCustody::OpenOnly => None,
                    RecoveredNativeCustody::Relational(native) => {
                        let (effect, before, before_lease, after, after_lease) =
                            native.consume(|native| native.into_retained_parts());
                        drop((before, before_lease, after, after_lease));
                        Some(effect)
                    }
                    RecoveredNativeCustody::Released => unreachable!("released exactly once"),
                    RecoveredNativeCustody::Continuing(_) | RecoveredNativeCustody::Working => {
                        unreachable!("the exclusive release refused continuing custody")
                    }
                };
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
