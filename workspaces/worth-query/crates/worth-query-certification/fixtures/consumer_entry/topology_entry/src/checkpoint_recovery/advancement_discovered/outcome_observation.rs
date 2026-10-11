//! Failure descriptions of the fixture's typed outcomes, without production Debug hooks.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome as RequiredOutcome;

pub(super) fn discovered(
    outcome: &Outcome<CheckpointSchema, PlanarSourceAdjustment, DiscoveredProgram, DiscoveredRoot>,
) -> String {
    match outcome {
        Outcome::ProductUnpublished(_) => "ProductUnpublished".into(),
        Outcome::Performed(performed) => format!("Performed({:?})", performed.receipt()),
        Outcome::RequiredOutputDenied {
            receipt, denial, ..
        } => format!("RequiredOutputDenied({receipt:?}, {denial:?})"),
        Outcome::Blocked(blocked) => format!("Blocked({:?})", blocked.outcome()),
        Outcome::NotPerformed(outcome) => format!("NotPerformed({outcome:?})"),
    }
}

pub(super) fn required(
    outcome: &RequiredOutcome<
        CheckpointSchema,
        PlanarSourceAdjustment,
        CheckpointProgram,
        CheckpointRoot,
    >,
) -> String {
    match outcome {
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
