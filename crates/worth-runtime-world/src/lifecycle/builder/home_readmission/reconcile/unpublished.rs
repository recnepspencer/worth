//! Original unfinished operations bind to their exact recovered branch reference.
use super::*;
use crate::durability::RecoveredCustodyDisposition;
use crate::identity::readmission::{readmit_attempt, readmit_commit, readmit_custody};
use crate::recovery::RecoveredWorldPublicationInputs;
use worth_relational::facade::{
    branch::RecordedRelationalComponentReference, runtime::RelationalRuntime,
};

pub(super) fn install<D, I, E, Ctx, T>(
    root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    binding: &Arc<WorldDurableMediumBinding>,
    image: &WorldRecoveredJournalState,
    native: &RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    effects: &super::super::native_effects::ReadmittedNativeEffects,
    signal_effects: &super::super::signal_effects::ReadmittedSignalEffects,
    descriptors: &ExecutionOrderedMap<
        worth_foundational::facade::DurableCompositeCommitIdentity,
        RecordedRelationalComponentReference,
    >,
    observations: &Observations,
    custody: &mut WorldNativeReadmissionCustody,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let owner = root.owner_identity();
    for open in image.opens.values() {
        if image
            .history
            .get(&open.commit)
            .is_some_and(|head| head.represents_open(open.open()))
        {
            continue;
        }
        let expected = find(observations, &open.expected)?;
        let disposition = image.dispositions.get(&open.association.attempt());
        if let Some(disposition) = disposition
            && let RecoveredCustodyDisposition::Released(_) = disposition.disposition()
        {
            root.state
                .recovery
                .readmit_released_original(
                    execution,
                    disposition,
                    expected,
                    effects.take(&open.association.attempt()),
                )
                .map_err(Denial::Medium)?;
            continue;
        }
        let before = descriptors
            .get(&open.expected.commit)
            .ok_or(Denial::MissingPredecessor)?;
        let admitted = match effects.take(&open.association.attempt()) {
            Some(effect) => {
                let after = authority
                    .recorded_publication_successor(execution, native, &effect)
                    .map_err(Denial::NativeSuccessor)?;
                Some(
                    authority
                        .admit_recorded_publication(execution, native, effect, before, &after)
                        .map_err(|cause| Denial::NativePublication(Box::new(cause)))?,
                )
            }
            None => None,
        };
        let index = custody.publications.len();
        custody.publications.push(admitted);
        let signal = signal_effects.get(&open.association.attempt());
        if custody.publications[index].is_none()
            && signal.is_none()
            && matches!(
                disposition.map(|row| row.disposition()),
                Some(RecoveredCustodyDisposition::Continued)
            )
        {
            return Err(Denial::OccurrenceEvidenceMismatch);
        }
        let planned_signal = outstanding_signal_leg(
            image,
            open,
            custody.publications[index].is_some(),
            signal.is_some(),
            disposition.map(|row| row.disposition()),
        )?;
        let attempt =
            readmit_attempt(owner, open.association.attempt()).ok_or(Denial::ForeignOwner)?;
        let commit = readmit_commit(owner, open.commit).ok_or(Denial::ForeignOwner)?;
        let identity = readmit_custody(owner, open.custody).ok_or(Denial::ForeignOwner)?;
        let native_identity = custody.publications[index]
            .as_ref()
            .map(|publication| {
                native
                    .history()
                    .immutable_commit_identity(
                        publication.custody().effect().subject().receipt().commit_id,
                    )
                    .ok_or(Denial::OccurrenceEvidenceMismatch)
            })
            .transpose()?;
        let slot = root
            .state
            .recovery
            .reserve_product_unpublished(owner)
            .map_err(|_| Denial::RecoveryCapacity)?;
        let slot_index = custody.slots.len();
        custody.slots.push(Some(slot));
        custody.slots[slot_index]
            .take()
            .expect("same reserved slot")
            .install_recovered(RecoveredWorldPublicationInputs {
                identity,
                attempt,
                commit,
                continued: disposition.cloned(),
                open: open.clone(),
                expected: expected.clone(),
                binding: binding.clone(),
                native_identity,
                native: custody.publications[index].take(),
                signal,
                planned_signal,
            });
    }
    Ok(())
}

/// A mixed original durably records open, planned Signal leg, Native's effect
/// and Signal's effect in that order. Open-only and open-with-leg carry no
/// owner effect; Native-only keeps its leg for continuation; any other mix
/// contradicts the order and refuses before service.
fn outstanding_signal_leg(
    image: &WorldRecoveredJournalState,
    open: &crate::durability::AdmittedWorldOpenAttempt,
    native: bool,
    signal: bool,
    disposition: Option<&RecoveredCustodyDisposition>,
) -> Result<Option<crate::durability::AdmittedWorldPlannedSignalLeg>, Denial> {
    use crate::durability::WorldOpenComponentIntent as Intent;
    if open.component_intent != Intent::RelationalAndSignal {
        return Ok(None);
    }
    let planned = image.planned_legs.get(&open.association.attempt());
    match (planned, native, signal) {
        (_, false, false) | (Some(_), true, true) => Ok(None),
        (Some(planned), true, false)
            if !matches!(disposition, Some(RecoveredCustodyDisposition::Continued)) =>
        {
            Ok(Some(planned.clone()))
        }
        (Some(_), true, false) | (None, true, _) | (_, false, true) => {
            Err(Denial::OccurrenceEvidenceMismatch)
        }
    }
}
