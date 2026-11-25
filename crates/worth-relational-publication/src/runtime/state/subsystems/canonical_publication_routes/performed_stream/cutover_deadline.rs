//! Framing cannot extend the caller's cutover narrowing.
use super::*;
use worth_relational_durability::internal_port::durability::canonical_commit_artifact::memory_publication::denial::MemoryFramePreparationDenial;
use worth_relational_durability::internal_port::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;

pub(super) fn require<'a, 'b>(
    admission: PatchPositionReservationAdmission<'_, '_, '_>,
    deadline: Option<std::time::Instant>,
    allow_effect: Option<&'a mut (dyn FnMut() -> bool + 'b)>,
) -> Result<(), CanonicalPublicationRecordError> {
    match admission {
        PatchPositionReservationAdmission::DeferWhenHeld => {}
        PatchPositionReservationAdmission::DeferBehindCommit(execution)
        | PatchPositionReservationAdmission::WaitWithin(execution) => {
            execution
                .consult()
                .map_err(CanonicalPublicationRecordError::ReservationWaitStopped)?;
        }
    }
    if deadline.is_some_and(|deadline| std::time::Instant::now() >= deadline) {
        return Err(CanonicalPublicationRecordError::CutoverDeadlineElapsed);
    }
    if allow_effect.is_some_and(|allow| !allow()) {
        return Err(CanonicalPublicationRecordError::CallerCutoffStopped);
    }
    Ok(())
}

pub(super) fn prepare_frame<'a, 'b>(
    admission: PatchPositionReservationAdmission<'_, '_, '_>,
    deadline: Option<std::time::Instant>,
    allow_effect: Option<&'a mut (dyn FnMut() -> bool + 'b)>,
    frame: impl FnOnce() -> Result<
        Option<RelationalCanonicalCommitArtifact>,
        MemoryFramePreparationDenial,
    >,
) -> Result<Option<RelationalCanonicalCommitArtifact>, CanonicalPublicationRecordError> {
    let original = frame().map_err(CanonicalPublicationRecordError::MemoryFrame)?;
    require(admission, deadline, allow_effect)?;
    Ok(original)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn frame_preparation_rechecks_deadline_before_position_admission() {
        let prepared = std::cell::Cell::new(false);
        let deadline = std::time::Instant::now();
        let result = prepare_frame(
            PatchPositionReservationAdmission::DeferWhenHeld,
            Some(deadline),
            None,
            || {
                prepared.set(true);
                Ok(None)
            },
        );
        assert!(
            prepared.get(),
            "the refusal is checked after frame preparation"
        );
        assert!(matches!(
            result,
            Err(CanonicalPublicationRecordError::CutoverDeadlineElapsed)
        ));
    }
}
