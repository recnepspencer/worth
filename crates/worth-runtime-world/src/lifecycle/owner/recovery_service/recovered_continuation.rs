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
        let prepared = effects
            .record
            .continuation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some();
        if !prepared {
            if let Err(cause) = self.prepare_recovered_continuation(execution, &mut effects) {
                return Outcome::Refused {
                    cause,
                    custody: effects,
                };
            }
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
            )) => native,
            Some(crate::recovery::recovered_publication::RecoveredNativeObservation::Prepared(
                _,
            )) => return Err(Denial::Admission(RuntimeWorldRecoveryDenial::Busy)),
            None => return Err(Denial::NoOwnerEffect),
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
        let (after, attempt) = self
            .prepare_original_recovered_image(
                effects.expected_head(),
                &effects.record.open,
                native,
                effects
                    .record
                    .native_identity
                    .clone()
                    .ok_or(Denial::NoOwnerEffect)?,
            )
            .map_err(|cause| Denial::Readmission(Box::new(cause)))?;
        let funding = execution
            .reserve_retained_memory(
                (std::mem::size_of::<PreparedRecoveredPublication>()
                    + std::mem::size_of::<crate::history::ReadmittedPublicationRecord>()
                    + std::mem::size_of::<crate::history::PreparedReadmittedPublicationRecord>()
                    + std::mem::size_of::<crate::history::CompositeRuntimeWorldCommit>()
                    + 6 * std::mem::size_of::<usize>()) as u64,
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
        let mut origin =
            PreparedCheckpointOccurrence::prepare(execution).map_err(Denial::Durable)?;
        let origin_pins = pins
            .fork_history(after.commit().basis())
            .map_err(|_| Denial::Durable(WorldDurableMediumDenial::CheckpointOriginRetention))?;
        origin.bind(after.commit().basis(), origin_pins);
        let transfer = pins
            .try_transfer_product_head(after.commit().basis())
            .map_err(|_| Denial::Durable(WorldDurableMediumDenial::CheckpointOriginRetention))?;
        let subject = native.custody().effect().subject();
        let occurrence = WorldDurableArtifact::from_record(
            WorldRecord::Occurrence {
                open: effects.record.open.open().clone(),
                relational: WorldRelationalEvidence {
                    commit: subject.receipt().commit_id.0,
                    position: subject.position().0,
                    digest: *subject.artifact_digest(),
                },
                publishing_incarnation: effects.claim_incarnation(),
            },
            execution,
        )
        .map_err(Denial::Durable)?;
        let disposition = match &effects.record.continued {
            Some(source) => OriginalContinuationDisposition::Readmitted(source.clone()),
            None => OriginalContinuationDisposition::Undisposed(
                WorldDurableArtifact::from_record(
                    WorldRecord::Continued {
                        association: effects.record.open.association.clone(),
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
        let RecoveredNativeCustody::Relational(native) = original else {
            unreachable!("exclusive original source phase")
        };
        let native = match Arc::try_unwrap(native) {
            Ok(native) => native,
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
        };
        let record = crate::history::ReadmittedPublicationRecord::prepare_original(
            effects.expected_head().snapshot().clone(),
            after,
            attempt,
            effects.claim_incarnation(),
            native,
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
