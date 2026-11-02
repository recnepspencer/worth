//! Original Native source eligibility precedes every descriptor/incarnation effect.
use super::ApplicationHomeOpenRefusal as Denial;
use crate::checkpoint::{SelectedHomeManifest, SelectedHomeSource};
use crate::home_context::RelationalHomeOpeningInput;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder,
};
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence, durability::RelationalDurableReadBounds,
    publication::PatchStreamPosition,
};
use worth_store::physical_runtime::{
    OrderedJournalName, RecoveryStorePresence, ServingPhysicalRuntime,
    RELATIONAL_BRANCH_TRANSITIONS, RELATIONAL_CANONICAL_COMMITS,
};

pub(super) fn prepare(
    serving: &Arc<ServingPhysicalRuntime>,
    origin: RecoveryStorePresence,
    source: &SelectedHomeSource,
    selected: Option<&SelectedHomeManifest>,
    request: &ExecutionRequest<'_, '_>,
    #[cfg(feature = "certification-test-authority")] observer: &mut Option<
        Box<
            dyn Fn(PatchStreamPosition, worth_store_relational::RelationalStoreColdReadCounters)
                + Send
                + Sync,
        >,
    >,
) -> Result<RelationalHomeOpeningInput, Denial> {
    use worth_store_relational::RelationalHomeSourcePreparation as Preparation;
    match origin {
        RecoveryStorePresence::ProvenAbsent => {
            if selected.is_some() {
                return Err(Denial::Relational(
                    worth_store_relational::RelationalSectionReadmissionDenial::MalformedSection,
                ));
            }
            #[cfg(not(feature = "certification-test-authority"))]
            let prepared = Preparation::from_absent_home_read(serving.clone(), request);
            #[cfg(feature = "certification-test-authority")]
            let prepared = match observer.take() {
                Some(observer) => Preparation::from_absent_home_read_with_read_observer_for_test(
                    serving.clone(),
                    observer,
                    request,
                ),
                None => Preparation::from_absent_home_read(serving.clone(), request),
            };
            return prepared
                .map(RelationalHomeOpeningInput::Fresh)
                .map_err(Denial::Relational);
        }
        RecoveryStorePresence::ExistingArtifacts => {}
    }
    #[cfg(not(feature = "certification-test-authority"))]
    let preparation = Preparation::from_original_home_read(
        serving.clone(),
        selected.map(|image| image.relational_read()),
        request,
    );
    #[cfg(feature = "certification-test-authority")]
    let preparation = match observer.take() {
        Some(observer) => Preparation::from_original_home_read_with_read_observer_for_test(
            serving.clone(),
            selected.map(|image| image.relational_read()),
            request,
            observer,
        ),
        None => Preparation::from_original_home_read(
            serving.clone(),
            selected.map(|image| image.relational_read()),
            request,
        ),
    };
    let preparation = preparation.map_err(Denial::Relational)?;
    let covered = preparation.coverage();
    let canonical_name = OrderedJournalName::new(RELATIONAL_CANONICAL_COMMITS)
        .expect("closed Native canonical journal");
    let transitions_name = OrderedJournalName::new(RELATIONAL_BRANCH_TRANSITIONS)
        .expect("closed Native transition journal");
    let canonical_head = source
        .journal_head(serving, canonical_name, request)
        .map_err(Denial::Checkpoint)?;
    let transition_head = source
        .journal_head(serving, transitions_name, request)
        .map_err(Denial::Checkpoint)?;
    let canonical = tail(
        source,
        serving,
        canonical_name,
        covered[0],
        canonical_head,
        256 << 20,
        request,
    )?;
    let transitions = tail(
        source,
        serving,
        transitions_name,
        covered[1],
        transition_head,
        1 << 20,
        request,
    )?;
    let initial = source
        .native_initial_inputs(serving, request)
        .map_err(Denial::Checkpoint)?;
    preparation
        .finish(
            initial,
            canonical,
            transitions,
            RelationalDurableReadBounds {
                selected_checkpoint: selected.is_some(),
                canonical: PatchStreamPosition(canonical_head),
                transitions: RelationalBranchTransitionSequence(transition_head),
            },
            request,
        )
        .map(RelationalHomeOpeningInput::Existing)
        .map_err(Denial::Relational)
}

fn tail(
    source: &SelectedHomeSource,
    serving: &ServingPhysicalRuntime,
    name: OrderedJournalName,
    covered: u64,
    head: u64,
    limit: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionArray<ExecutionImmutableBytes>, Denial> {
    let count = head
        .checked_sub(covered)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or(Denial::Checkpoint(
            crate::ApplicationHomeCheckpointDenial::CoverageAheadOfJournal,
        ))?;
    let mut entries = ExecutionRequestArrayBuilder::allocate(*request, count).map_err(|cause| {
        Denial::Checkpoint(crate::ApplicationHomeCheckpointDenial::Payload(cause))
    })?;
    for offset in 0..count {
        let bytes = source
            .native_journal_payload(serving, name, covered + offset as u64 + 1, limit, request)
            .map_err(Denial::Checkpoint)?;
        entries.push(bytes).map_err(|cause| {
            Denial::Checkpoint(crate::ApplicationHomeCheckpointDenial::Payload(cause))
        })?;
    }
    entries
        .seal()
        .map_err(|cause| Denial::Checkpoint(crate::ApplicationHomeCheckpointDenial::Payload(cause)))
}
