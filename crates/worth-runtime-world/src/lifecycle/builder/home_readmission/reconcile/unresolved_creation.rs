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
    RecoveredCreationFate, RecoveredCreationForks, RecoveredWorldCreationInputs,
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
        // A continuing original's admitted fork stays in the refusal custody
        // until the same record takes it.
        let decided = match classify(
            intent,
            owners.forks.get(attempt).cloned(),
            owners.signal.creation(attempt),
        )? {
            RecordedCreationFate::Release => None,
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
                Some((custody.creation_forks.len() - 1, signal))
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
            None => RecoveredCreationFate::Release,
            Some((index, signal)) => RecoveredCreationFate::Continue(RecoveredCreationForks {
                native: custody.creation_forks[index].take(),
                signal,
            }),
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

/// No owner effect releases; every required fork continues; a partial
/// creation has no admissible decision without its lost live source permit.
fn classify(
    intent: &WorldCreationIntentRecord,
    native: Option<WorldRelationalForkEvidence>,
    signal: Option<Arc<SignalReadmittedSourcePoint>>,
) -> Result<RecordedCreationFate, Denial> {
    let native_forks = matches!(intent.relational, NativePlan::ForkExact { .. });
    let signal_forks = matches!(intent.signal, SignalPlan::ForkExact { .. });
    match (native.is_some(), signal.is_some()) {
        (false, false) => Ok(RecordedCreationFate::Release),
        (native_done, signal_done)
            if native_done == native_forks && signal_done == signal_forks =>
        {
            Ok(RecordedCreationFate::Continue { native, signal })
        }
        (_, _) => Err(Denial::PartlyPerformedCreation),
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
