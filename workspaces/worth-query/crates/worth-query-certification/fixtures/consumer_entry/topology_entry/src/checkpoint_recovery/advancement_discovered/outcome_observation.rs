//! Failure descriptions of the fixture's typed outcomes, without production Debug hooks.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome as RequiredOutcome;

pub(super) fn discovered(
    outcome: &Outcome<CheckpointSchema, PlanarSourceAdjustment, DiscoveredProgram, DiscoveredRoot>,
) -> String {
    match outcome {
        Outcome::CleanupRequired { cleanup, .. } => {
            format!("CleanupRequired({:?})", cleanup.failure().kind())
        }
        Outcome::ProductUnpublished(_) => "ProductUnpublished".into(),
        Outcome::Performed(performed) => format!("Performed({:?})", performed.receipt()),
        Outcome::RequiredOutputDenied(failure) => format!(
            "RequiredOutputDenied({:?}, {:?})",
            failure.receipt(),
            failure.denial()
        ),
        Outcome::Blocked(blocked) => format!("Blocked({:?})", blocked.outcome()),
        Outcome::NotPerformed(outcome) => format!("NotPerformed({outcome:?})"),
    }
}

pub(super) fn required(
    outcome: &RequiredOutcome<
        CheckpointSchema,
        crate::PlanarSourceAdjustmentBinding<CheckpointSchema>,
        CheckpointProgram,
        CheckpointRoot,
    >,
) -> String {
    match outcome {
        RequiredOutcome::CleanupRequired { cleanup, .. } => {
            format!("CleanupRequired({:?})", cleanup.failure().kind())
        }
        RequiredOutcome::ProductUnpublished(_) => "ProductUnpublished".into(),
        RequiredOutcome::Performed(performed) => format!("Performed({:?})", performed.receipt()),
        RequiredOutcome::RequiredOutputDenied(failure) => format!(
            "RequiredOutputDenied({:?}, {:?})",
            failure.receipt(),
            failure.denial()
        ),
        RequiredOutcome::Blocked(blocked) => format!("Blocked({:?})", blocked.outcome()),
        RequiredOutcome::NotPerformed(outcome) => format!("NotPerformed({outcome:?})"),
    }
}
