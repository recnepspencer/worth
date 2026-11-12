//! Pre-admit original history, pins and the exact authoritative cell movement.
use super::super::RuntimeWorldOwnerRoot;
use crate::recovery::recovered_publication::continuation::OriginalContinuationDisposition;
use crate::{durability::*, recovery::*};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn continue_recovered_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        mut effects: WorldRecoveredUnpublishedEffects,
    ) -> WorldRecoveredContinuationOutcome {
        use WorldRecoveredContinuationDenial as Denial;
        use WorldRecoveredContinuationOutcome as Outcome;
        if !self.state.recovery.admits_recovered(&effects) {
            return Outcome::Refused {
                cause: Denial::Admission(RuntimeWorldRecoveryDenial::ForeignHandle),
                custody: effects,
            };
        }
        let _operation = match self.reserve_recovery_operation_if_open_and_bootstrapped() {
            Ok(operation) => operation,
            Err(()) => {
                return Outcome::Refused {
                    cause: Denial::Admission(
                        crate::lifecycle::RuntimeWorldOwnerUnavailable::new().into(),
                    ),
                    custody: effects,
                }
            }
        };
        if effects
            .record
            .release
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some()
        {
            return Outcome::Refused {
                cause: Denial::Admission(RuntimeWorldRecoveryDenial::ReleaseAlreadyStarted),
                custody: effects,
            };
        }
        if effects
            .native_effect()
            .and_then(|effect| effect.subject().association())
            .is_some_and(|value| {
                worth_foundational::facade::HomeProgramTransitionCaller::recognizes(
                    value.as_bytes(),
                )
            })
        {
            return Outcome::Refused {
                cause: Denial::Admission(RuntimeWorldRecoveryDenial::HomeProgramTransitionRequired),
                custody: effects,
            };
        }
        let prepared = effects
            .record
            .continuation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some();
        if !prepared
            && let Err(cause) = self.prepare_recovered_continuation(execution, &mut effects)
        {
            return Outcome::Refused {
                cause,
                custody: effects,
            };
        }
        effects.advance_original(execution, &self.state.operation_control)
    }
    fn prepare_recovered_continuation(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        effects: &mut WorldRecoveredUnpublishedEffects,
    ) -> Result<(), WorldRecoveredContinuationDenial> {
        use WorldRecoveredContinuationDenial as Denial;
        execution
            .consult()
            .map_err(|cause| Denial::Durable(WorldDurableMediumDenial::Request(cause)))?;
        let native = match effects.source.as_ref() {
            Some(crate::recovery::recovered_publication::RecoveredNativeObservation::Original(
                native,
            )) => Some(native),
            Some(crate::recovery::recovered_publication::RecoveredNativeObservation::Prepared(
                _,
            )) => return Err(Denial::Admission(RuntimeWorldRecoveryDenial::Busy)),
            None => None,
        };
        let cell = self
            .state
            .branches
            .branch_cell(effects.expected_head().branch_identity())
            .ok_or(Denial::PredecessorChanged)?;
        let actual = cell
            .observe(&self.state.history, &self.state.retention)
            .map_err(|_| Denial::PredecessorChanged)?;
        if actual.snapshot() != effects.expected_head().snapshot() {
            return Err(Denial::PredecessorChanged);
        }
        use crate::history::RecoveredWorldOwnerEvidenceRef as Owners;
        let owners = match (native, effects.record.signal.as_deref()) {
            (Some(native), None) => Owners::RelationalOnly(native.custody()),
            (None, Some(signal)) => Owners::SignalOnly(signal),
            (Some(native), Some(signal)) => Owners::RelationalAndSignal {
                relational: native.custody(),
                signal,
            },
            (None, None) => return Err(Denial::NoOwnerEffect),
        };
        use crate::durability::WorldOpenComponentIntent as Intent;
        let actual_intent = match owners {
            Owners::RelationalOnly(_) => Intent::RelationalOnly,
            Owners::SignalOnly(_) => Intent::SignalOnly,
            Owners::RelationalAndSignal { .. } => Intent::RelationalAndSignal,
        };
        if actual_intent != effects.record.open.component_intent {
            return Err(Denial::OwnerEffectsMismatch {
                expected: effects.record.open.component_intent,
                actual: actual_intent,
            });
        }
        let prepared = self
            .prepare_readmitted_successor(
                execution,
                effects.expected_head(),
                &effects.record.open,
                owners,
                effects
                    .expected_head()
                    .basis()
                    .correspondence_basis()
                    .clone(),
            )
            .map_err(|cause| Denial::Readmission(Box::new(cause)))?;
        let after = prepared.after;
        let attempt = prepared.attempt;
        let record_backing = prepared.record;
        let funding = execution
            .reserve_retained_memory(
                (std::mem::size_of::<PreparedRecoveredPublication>()
                    + std::mem::size_of::<crate::history::PreparedReadmittedPublicationRecord>()
                    + std::mem::size_of::<crate::history::CompositeRuntimeWorldCommit>()
                    + 4 * std::mem::size_of::<usize>()) as u64,
            )
            .map_err(|cause| Denial::Durable(WorldDurableMediumDenial::Request(cause)))?;
        let capacity = self
            .state
            .history
            .reserve_commit_capacity(
                after.selected_commit().clone(),
                after.commit().parent().clone(),
            )
            .map_err(Denial::History)?;
        let mut pair = self
            .state
            .retention
            .reserve_product_publication_pair()
            .map_err(Denial::Retention)?;
        let mut pins = pair
            .try_bind_publication(after.commit().basis())
            .map_err(Denial::Retention)?;
        let history_pins = pins
            .fork_history(after.commit().basis())
            .map_err(|_| Denial::Durable(WorldDurableMediumDenial::CheckpointOriginRetention))?;
        let origin_pins = pins
            .fork_history(after.commit().basis())
            .map_err(|_| Denial::Durable(WorldDurableMediumDenial::CheckpointOriginRetention))?;
        let transfer = pins
            .try_transfer_product_head(after.commit().basis())
            .map_err(|_| Denial::Durable(WorldDurableMediumDenial::CheckpointOriginRetention))?;
        let relational = owners.relational().map(|native| {
            let subject = native.effect().subject();
            WorldRelationalEvidence {
                commit: subject.receipt().commit_id.0,
                position: subject.position().0,
                digest: *subject.artifact_digest(),
            }
        });
        let signal = owners
            .signal()
            .map(|signal| WorldSignalEvidence::from_point(signal.source_point().clone(), execution))
            .transpose()
            .map_err(Denial::Durable)?;
        let evidence = match (relational, signal) {
            (Some(relational), None) => WorldOccurrenceOwnerEvidence::RelationalOnly(relational),
            (None, Some(signal)) => WorldOccurrenceOwnerEvidence::SignalOnly(signal),
            (Some(relational), Some(signal)) => {
                WorldOccurrenceOwnerEvidence::RelationalAndSignal { relational, signal }
            }
            (None, None) => return Err(Denial::NoOwnerEffect),
        };
        let original_open =
            WorldDurableArtifact::from_readmitted_open(&effects.record.open, execution)
                .map_err(Denial::Durable)?;
        let mut original_occurrence =
            WorldOccurrenceCustody::prepare(&original_open, execution).map_err(Denial::Durable)?;
        original_occurrence.bind_origin(after.commit().basis(), origin_pins);
        let occurrence = original_occurrence
            .finalize_owners(evidence, effects.claim_incarnation())
            .map_err(Denial::Durable)?
            .clone();
        let origin = original_occurrence.take_origin();
        let disposition = match &effects.record.continued {
            Some(source) => OriginalContinuationDisposition::Readmitted(source.clone()),
            None => OriginalContinuationDisposition::Undisposed(
                WorldDurableArtifact::from_input(
                    WorldArtifactInput::Continued {
                        association: &effects.record.open.association,
                        custody: effects.record.open.custody,
                    },
                    execution,
                )
                .map_err(Denial::Durable)?,
            ),
        };
        let permit = cell
            .begin_durable_movement(effects.expected_head(), occurrence.clone())
            .map_err(|_| Denial::PredecessorChanged)?;
        // All fallible preparation precedes transfer of the SAME original Native proof.
        effects.source = None;
        let original = std::mem::replace(
            &mut *effects
                .record
                .native
                .lock()
                .unwrap_or_else(|e| e.into_inner()),
            RecoveredNativeCustody::Released,
        );
        let native = match original {
            RecoveredNativeCustody::OpenOnly => None,
            RecoveredNativeCustody::Relational(native) => match Arc::try_unwrap(native) {
                Ok(native) => Some(native),
                Err(native) => {
                    *effects
                        .record
                        .native
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) =
                        RecoveredNativeCustody::Relational(native);
                    permit.proven_no_effect();
                    effects.refresh_source();
                    return Err(Denial::Admission(RuntimeWorldRecoveryDenial::Busy));
                }
            },
            RecoveredNativeCustody::Prepared(_)
            | RecoveredNativeCustody::Continuing(_)
            | RecoveredNativeCustody::Working
            | RecoveredNativeCustody::Released => {
                unreachable!("the original source phase was admitted before preparation")
            }
        };
        let owners = crate::history::RecoveredWorldOwnerEvidence::from_original_components(
            native,
            effects.record.signal.clone(),
        )
        .expect("the admitted original has an owner effect");
        let record = crate::history::ReadmittedPublicationRecord::prepare_original(
            record_backing,
            effects.expected_head().snapshot().clone(),
            after,
            attempt,
            effects.claim_incarnation(),
            owners,
        );
        *effects
            .record
            .native
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = RecoveredNativeCustody::Prepared(record.clone());
        *effects
            .record
            .continuation
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(Box::new(PreparedRecoveredPublication {
            record,
            sealed: None,
            capacity,
            pins: Some(history_pins),
            transfer: Some(transfer),
            protection: None,
            origin: Some(origin),
            disposition,
            occurrence,
            committed: None,
            permit: Some(permit),
            _funding: funding,
        }));
        effects.refresh_source();
        Ok(())
    }
}
