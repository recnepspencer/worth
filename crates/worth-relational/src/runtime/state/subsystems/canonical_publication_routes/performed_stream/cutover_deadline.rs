//! Framing cannot extend the caller's narrowing or the prepared candidate's life.
use super::*;
use crate::durability::canonical_commit_artifact::{
    MemoryFramePreparationDenial, RelationalCanonicalCommitArtifact,
};

pub(super) fn require(
    admission: PatchPositionReservationAdmission<'_, '_, '_>,
    deadline: Option<std::time::Instant>,
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
    Ok(())
}

pub(super) fn prepare_frame(
    admission: PatchPositionReservationAdmission<'_, '_, '_>,
    deadline: Option<std::time::Instant>,
    frame: impl FnOnce() -> Result<
        Option<RelationalCanonicalCommitArtifact>,
        MemoryFramePreparationDenial,
    >,
) -> Result<Option<RelationalCanonicalCommitArtifact>, CanonicalPublicationRecordError> {
    let original = frame().map_err(CanonicalPublicationRecordError::MemoryFrame)?;
    require(admission, deadline)?;
    Ok(original)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame_preparation_rechecks_deadline_before_position_admission() {
        let prepared = std::cell::Cell::new(false);
        let deadline = std::time::Instant::now();
        let result = prepare_frame(
            PatchPositionReservationAdmission::DeferWhenHeld,
            Some(deadline),
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
