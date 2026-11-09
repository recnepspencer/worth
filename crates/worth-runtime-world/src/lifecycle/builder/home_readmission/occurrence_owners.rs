//! Each cold occurrence obtains its exact acting owners from protected originals.
use crate::durability::{AdmittedWorldOccurrence, WorldReadmissionDenial as Denial};
use crate::history::RecoveredWorldOwnerEvidence;
use crate::lifecycle::RuntimeWorldAuthoredSignalAttachment;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::RecordedRelationalComponentReference, durability::RecoveredRelationalRuntimeAuthority,
    runtime::RelationalRuntime,
};
pub(super) fn readmit<D, I, E, Ctx, T>(
    occurrence: &AdmittedWorldOccurrence,
    before: &RecordedRelationalComponentReference,
    signal: &RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>,
    native: &RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    effects: &super::native_effects::ReadmittedNativeEffects,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    (
        RecoveredWorldOwnerEvidence,
        RecordedRelationalComponentReference,
    ),
    Denial,
>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    let open = occurrence.open();
    let signal = match occurrence.owners().signal() {
        Some(recorded) => {
            let actual = signal
                .source_port()
                .readmit_source_point(recorded.point(), request)
                .map_err(|cause| Denial::SignalSource(Box::new(cause)))?;
            if actual.original().association() != Some(&open.association) {
                return Err(Denial::OccurrenceEvidenceMismatch);
            }
            Some(actual)
        }
        None => None,
    };
    let (native, after) = match occurrence.relational() {
        Some(recorded) => {
            let effect = effects
                .take(&open.association.attempt())
                .ok_or(Denial::OccurrenceEvidenceMismatch)?;
            let subject = effect.subject();
            if subject.receipt().commit_id.0 != recorded.commit
                || subject.position().0 != recorded.position
                || subject.artifact_digest() != &recorded.digest
            {
                return Err(Denial::OccurrenceEvidenceMismatch);
            }
            let after = authority
                .recorded_publication_successor(request, native, &effect)
                .map_err(Denial::NativeSuccessor)?;
            let actual = authority
                .admit_recorded_publication(request, native, effect, before, &after)
                .map_err(|cause| Denial::NativePublication(Box::new(cause)))?;
            (Some(actual), after)
        }
        None => {
            if effects.take(&open.association.attempt()).is_some() {
                return Err(Denial::OccurrenceEvidenceMismatch);
            }
            (None, before.clone())
        }
    };
    let owners = RecoveredWorldOwnerEvidence::from_admitted_components(native, signal)
        .ok_or(Denial::OccurrenceEvidenceMismatch)?;
    Ok((owners, after))
}
