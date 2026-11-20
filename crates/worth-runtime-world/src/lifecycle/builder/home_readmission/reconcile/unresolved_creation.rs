//! An unresolved original creation readmits as the SAME recovered custody:
//! its recorded identities, source and target name, with the fate decided by
//! each owner's own recovered inventory. No replacement identity is issued.
use super::*;
use crate::branch::SignalBranchCreationPlan as SignalPlan;
use crate::durability::{
    encode_association, WorldCreationIntentRecord,
    WorldRecordedRelationalCreationPlan as NativePlan, WorldRelationalForkEvidence,
};
use crate::identity::readmission::readmit_custody;
use crate::recovery::{
    RecoveredCreationFate, RecoveredCreationForks, RecoveredSignalForkCompletion,
    RecoveredSignalForkPhase, RecoveredWorldCreationInputs,
};
use worth_relational::facade::{
    durability::ReadmittedRelationalAssociatedFork, mvcc::RelationalCallerAssociation,
    runtime::RelationalRuntime,
};
use worth_signal::facade::durability::SignalReadmittedSourcePoint;

/// Each owner's complete recovered inventory of unresolved creation effects.
pub(super) struct OwnerInventories<'a> {
    pub(super) forks: &'a super::super::creation_effects::UnresolvedCreationForks,
    pub(super) signal: &'a super::super::signal_effects::ReadmittedSignalEffects,
}

/// The recovered references every original creation names as its source.
pub(super) struct RecoveredReferences<'a> {
    pub(super) observations: &'a Observations,
    pub(super) descriptors: &'a Descriptors,
    pub(super) points: &'a Points,
}

/// What the owners' inventories admit, before any owner admission runs.
enum RecordedCreationFate {
    Release,
    Continue {
        native: Option<WorldRelationalForkEvidence>,
        signal: Option<Arc<SignalReadmittedSourcePoint>>,
    },
    CompleteSignalFork {
        native: WorldRelationalForkEvidence,
    },
}

/// What readmission holds for the record while its admitted fork, if any,
/// stays in the refusal custody at `fork`.
enum AdmittedCreationFate {
    Release,
    Continue {
        fork: usize,
        signal: Option<Arc<SignalReadmittedSourcePoint>>,
    },
    CompleteSignalFork {
        fork: usize,
    },
}

pub(super) fn install<D, I, E, Ctx, T>(
    root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    binding: &Arc<WorldDurableMediumBinding>,
    image: &WorldRecoveredJournalState,
    native: &RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    owners: &OwnerInventories<'_>,
    references: &RecoveredReferences<'_>,
    custody: &mut WorldNativeReadmissionCustody,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let owner = root.owner_identity();
    for (attempt, original) in &image.creations {
        if image.created.contains_key(attempt) || image.creation_releases.contains_key(attempt) {
            continue;
        }
        let intent = original.intent();
        let source = find(references.observations, &intent.source)?;
        let source_signal = references
            .points
            .get(&(intent.source.incarnation, intent.source.generation))
            .ok_or(Denial::MissingPredecessor)?;
        // A performed original's admitted fork stays in the refusal custody
        // until the same record takes it.
        let decided = match classify(
            intent,
            owners.forks.get(attempt).cloned(),
            owners.signal.creation(attempt),
        )? {
            RecordedCreationFate::Release => AdmittedCreationFate::Release,
            RecordedCreationFate::Continue {
                native: fork,
                signal,
            } => {
                let admitted = match fork {
                    Some(fork) => Some(admit_fork(
                        root, native, authority, intent, &fork, references, execution,
                    )?),
                    None => None,
                };
                custody.creation_forks.push(admitted);
                AdmittedCreationFate::Continue {
                    fork: custody.creation_forks.len() - 1,
                    signal,
                }
            }
            RecordedCreationFate::CompleteSignalFork { native: fork } => {
                let admitted = admit_fork(
                    root, native, authority, intent, &fork, references, execution,
                )?;
                custody.creation_forks.push(Some(admitted));
                AdmittedCreationFate::CompleteSignalFork {
                    fork: custody.creation_forks.len() - 1,
                }
            }
        };
        let identity = readmit_custody(owner, intent.custody).ok_or(Denial::ForeignOwner)?;
        let destination = super::creation::reserve_destination(root, intent)?;
        let slot = root
            .state
            .recovery
            .reserve_product_unpublished(owner)
            .map_err(|_| Denial::RecoveryCapacity)?;
        let fate = match decided {
            AdmittedCreationFate::Release => RecoveredCreationFate::Release,
            AdmittedCreationFate::Continue { fork, signal } => {
                RecoveredCreationFate::Continue(RecoveredCreationForks {
                    native: custody.creation_forks[fork].take(),
                    signal,
                })
            }
            AdmittedCreationFate::CompleteSignalFork { fork } => {
                RecoveredCreationFate::CompleteSignalFork(RecoveredSignalForkCompletion {
                    native: custody.creation_forks[fork]
                        .take()
                        .expect("the admitted Native fork"),
                    phase: RecoveredSignalForkPhase::Unbegun,
                })
            }
        };
        slot.install_recovered_creation(RecoveredWorldCreationInputs {
            identity,
            intent: intent.clone(),
            source: source.clone(),
            source_signal: source_signal.clone(),
            binding: binding.clone(),
            fate,
            destination,
        });
    }
    Ok(())
}

/// No owner effect releases; every required fork continues. The live driver
/// makes Native's fork durable before Signal's, so a kill between them leaves
/// only Signal's planned fork outstanding: a performed effect that continues.
/// A durable Signal fork without Native's contradicts that order.
fn classify(
    intent: &WorldCreationIntentRecord,
    native: Option<WorldRelationalForkEvidence>,
    signal: Option<Arc<SignalReadmittedSourcePoint>>,
) -> Result<RecordedCreationFate, Denial> {
    let native_forks = matches!(intent.relational, NativePlan::ForkExact { .. });
    let signal_forks = matches!(intent.signal, SignalPlan::ForkExact { .. });
    match (native, signal) {
        (None, None) => Ok(RecordedCreationFate::Release),
        (native, signal)
            if native.is_some() == native_forks && signal.is_some() == signal_forks =>
        {
            Ok(RecordedCreationFate::Continue { native, signal })
        }
        (Some(native), None) if native_forks && signal_forks => {
            Ok(RecordedCreationFate::CompleteSignalFork { native })
        }
        (_, _) => Err(Denial::CreationEvidenceMismatch),
    }
}

/// Native describes its fork target, then proves it is exactly the recorded
/// fork of the recorded source component under the original association.
fn admit_fork<D, I, E, Ctx, T>(
    root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    native: &RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    intent: &WorldCreationIntentRecord,
    fork: &WorldRelationalForkEvidence,
    references: &RecoveredReferences<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<ReadmittedRelationalAssociatedFork, Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let NativePlan::ForkExact { target, .. } = &intent.relational else {
        return Err(Denial::CreationEvidenceMismatch);
    };
    let before = references
        .descriptors
        .get(&intent.source.commit)
        .ok_or(Denial::MissingPredecessor)?;
    let identity = native
        .branch_identity(target)
        .map_err(|_| Denial::CreationEvidenceMismatch)?;
    let (_, target_basis) = native
        .observe_branch(&identity)
        .map_err(|_| Denial::CreationEvidenceMismatch)?;
    let after = root
        .state
        .relational_services()
        .basis_port()
        .export_recorded_retained_component(execution, &target_basis)
        .map_err(Denial::NativeRecorded)?;
    drop(target_basis);
    let bytes = encode_association(execution, &intent.association).map_err(Denial::Medium)?;
    let association = RelationalCallerAssociation::from_bytes(bytes.elements())
        .map_err(|_| Denial::MalformedNativeAssociation)?;
    authority
        .admit_recorded_fork(
            execution,
            native,
            fork.sequence,
            &association,
            &fork.digest,
            before,
            &after,
        )
        .map_err(|cause| Denial::NativeFork(Box::new(cause)))
}
