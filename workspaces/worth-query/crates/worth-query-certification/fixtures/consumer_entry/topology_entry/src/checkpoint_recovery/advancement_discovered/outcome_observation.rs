//! Failure descriptions of the fixture's typed outcomes, without production Debug hooks.
use super::*;

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
