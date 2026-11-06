use super::release_preparation::ReleaseDestinationLease;
use super::*;
use crate::durability::{
    WorldArtifactInput, WorldDurableMediumDenial, WorldDurablePublicationStop,
};
use crate::recovery::{ProductUnpublishedCleanup, WorldReleasedEffects};

#[derive(Debug)]
pub enum WorldRecoveredReleaseOutcome {
    AdmissionRefused {
        cause: crate::recovery::RuntimeWorldRecoveryDenial,
        custody: WorldRecoveredUnpublishedEffects,
    },
    Released(WorldReleasedEffects),
    Pending(WorldRecoveredUnpublishedEffects),
    Refused {
        cause: WorldDurableMediumDenial,
        custody: WorldRecoveredUnpublishedEffects,
    },
    EvidenceRequired(WorldRecoveredUnpublishedEffects),
    Indeterminate(WorldRecoveredUnpublishedEffects),
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
        let transition_refusal = {
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
                        .is_some_and(|value| {
                            worth_foundational::facade::HomeProgramTransitionCaller::recognizes(
                                value.as_bytes(),
                            )
                        }) =>
                {
                    Some(crate::recovery::RuntimeWorldRecoveryDenial::HomeProgramTransitionRequired)
                }
                RecoveredNativeCustody::OpenOnly
                | RecoveredNativeCustody::Relational(_)
                | RecoveredNativeCustody::Prepared(_)
                | RecoveredNativeCustody::Released => None,
            }
        };
        if let Some(cause) = transition_refusal {
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
        let mut destination = match ReleaseDestinationLease::begin(execution, &self, &subject) {
            Ok(destination) => destination,
            Err(cause) => {
                return WorldRecoveredReleaseOutcome::Refused {
                    cause,
                    custody: self,
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
                        .unwrap_or_else(|e| e.into_inner()) = Some(committed);
                })
        };
        match publication {
            Ok(()) => {
                {
                    let committed = self
                        .record
                        .committed_release
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    destination.acknowledge(
                        committed
                            .as_ref()
                            .expect("same actual acknowledged release"),
                    );
                }
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
                    RecoveredNativeCustody::Continuing(_) | RecoveredNativeCustody::Working => {
                        unreachable!("the exclusive release refused continuing custody")
                    }
                };
                let original = destination.finish(effect);
                self.catalog.remove_recovered(&self.record);
                WorldRecoveredReleaseOutcome::Released(WorldReleasedEffects::new(
                    original,
                    ProductUnpublishedCleanup::new(Vec::new(), Vec::new()),
                ))
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
