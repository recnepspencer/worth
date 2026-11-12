//! Original-cut version scans precede every uncovered body admission.
use super::*;
use worth_relational::facade::durability::{
    RelationalBranchTransitionArtifact as Transition,
    RelationalCanonicalCommitArtifact as Canonical, RelationalDurableOwnedReadRefusal as Source,
    RelationalInitialInputsRefusal as Input,
};
pub(super) fn canonical(
    source: &SelectedHomeSource,
    serving: &ServingPhysicalRuntime,
    name: OrderedJournalName,
    covered: u64,
    head: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    scan(source, serving, name, covered, head, request, |bytes| {
        Canonical::preflight_frame_version(bytes).map_err(Source::CanonicalFrame)
    })
}
pub(super) fn transitions(
    source: &SelectedHomeSource,
    serving: &ServingPhysicalRuntime,
    name: OrderedJournalName,
    covered: u64,
    head: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    scan(source, serving, name, covered, head, request, |bytes| {
        Transition::preflight_frame_version(bytes).map_err(Source::TransitionFrame)
    })
}
fn scan(
    source: &SelectedHomeSource,
    serving: &ServingPhysicalRuntime,
    name: OrderedJournalName,
    covered: u64,
    head: u64,
    request: &ExecutionRequest<'_, '_>,
    validate: impl Fn(&[u8]) -> Result<(), Source>,
) -> Result<(), Denial> {
    if covered > head {
        return Err(Denial::Checkpoint(
            crate::ApplicationHomeCheckpointDenial::CoverageAheadOfJournal,
        ));
    }
    for position in covered..head {
        let bytes = source
            .native_journal_prefix(serving, name, position + 1, request)
            .map_err(Denial::Checkpoint)?;
        validate(&bytes).map_err(|cause| {
            Denial::Relational(
                worth_store_relational::RelationalSectionReadmissionDenial::InitialInputs(
                    Input::Source(cause),
                ),
            )
        })?;
    }
    Ok(())
}
