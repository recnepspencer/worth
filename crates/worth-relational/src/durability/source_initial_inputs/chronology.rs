//! Validate normalized original roles against the unchanged selected directory.
use super::{image::*, normalized::*, RelationalInitialInputsRefusal as Refusal};
use crate::branch::{
    RelationalBranchLifecyclePosture as Lifecycle, RelationalBranchTransition as Transition,
};
use crate::durability::data::RelationalCheckpointManifest;
use crate::history::data::BranchId;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

mod axes;
use axes::Axes;

struct Fold<'a> {
    current: ExecutionOrderedMap<&'a BranchId, Axes<'a>>,
    observations: ExecutionOrderedMap<(&'a BranchId, u64, u8), Axes<'a>>,
}
pub(super) fn validate(
    initial: &InitialInputsImage,
    normalized: &NormalizedOriginalInputs,
    manifest: &RelationalCheckpointManifest,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    let Some(InitialInputEntry::Birth(birth)) = initial.entries.first() else {
        return Err(Refusal::Malformed);
    };
    let mut fold = Fold {
        current: Default::default(),
        observations: Default::default(),
    };
    fold.record(&initial.main, Axes::empty(birth), request)?;
    let mut transition = 0usize;
    let mut installed = false;
    for position in 0..=normalized.canonical.len() {
        fold.install(initial, position, transition, &mut installed, request)?;
        while let Some(entry) = normalized.transitions.get(transition) {
            if entry.follows.0 != position as u64 {
                break;
            }
            fold.transition(&initial.main, entry, request)?;
            transition += 1;
            fold.install(initial, position, transition, &mut installed, request)?;
        }
        let Some(record) = normalized.canonical.get(position) else {
            break;
        };
        let index = manifest.commits.get(position).ok_or(Refusal::Malformed)?;
        let branch = &index.commit.branch_id;
        let mut state = fold
            .current
            .get(&branch)
            .cloned()
            .ok_or(Refusal::Malformed)?;
        if state.removed
            || state.lifecycle != Lifecycle::Live
            || state.schema != record.prestate_schema
        {
            return Err(Refusal::Malformed);
        }
        state.generation = state
            .generation
            .checked_advance()
            .map_err(|_| Refusal::Malformed)?;
        state.truth = state.truth.checked_advance().ok_or(Refusal::Malformed)?;
        state.commit = Some(index);
        state.schema = record.result_schema;
        fold.record(branch, state, request)?;
    }
    if transition != normalized.transitions.len() || installed != (initial.entries.len() == 2) {
        return Err(Refusal::Malformed);
    }
    if manifest.branch_cells.len()
        != fold
            .current
            .iter()
            .filter(|(_, state)| !state.removed)
            .count()
        || manifest.branch_lifecycle.len() != manifest.branch_cells.len()
    {
        return Err(Refusal::Malformed);
    }
    for (cell, lifecycle) in manifest.branch_cells.iter().zip(&manifest.branch_lifecycle) {
        let state = fold
            .current
            .get(&&cell.branch_id)
            .ok_or(Refusal::Malformed)?;
        if state.removed
            || state.generation != cell.observation.generation()
            || state.truth != cell.truth_version
            || !state.matches_target(cell.observation.target())
            || lifecycle.branch_id != cell.branch_id
            || lifecycle.posture != state.lifecycle
        {
            return Err(Refusal::Malformed);
        }
    }
    if manifest.retired_branch_names.len()
        != fold
            .current
            .iter()
            .filter(|(_, state)| state.lifecycle == Lifecycle::Deleting)
            .count()
        || manifest.retired_branch_names.iter().any(|branch| {
            fold.current
                .get(&branch)
                .is_none_or(|state| state.lifecycle != Lifecycle::Deleting)
        })
    {
        return Err(Refusal::Malformed);
    }
    for root in &manifest.roots {
        for descriptor in &root.recorded_components {
            fold.require_component(descriptor, root.schema_carrier_digest)?;
        }
    }
    for empty in &manifest.retained_empty_components {
        fold.require_component(&empty.descriptor, empty.schema_carrier_digest)?;
    }
    Ok(())
}
impl<'a> Fold<'a> {
    fn record(
        &mut self,
        branch: &'a BranchId,
        state: Axes<'a>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        self.observations
            .insert_cloned_native_for_request(
                &(
                    branch,
                    state.generation.get(),
                    axes::posture(state.lifecycle),
                ),
                &state,
                *request,
            )
            .map_err(map)?;
        self.current
            .insert_cloned_native_for_request(&branch, &state, *request)
            .map_err(map)
    }
    fn install(
        &mut self,
        initial: &'a InitialInputsImage,
        position: usize,
        transition: usize,
        installed: &mut bool,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        let Some(InitialInputEntry::Installation {
            before,
            after,
            follows_transition,
        }) = initial.entries.get(1)
        else {
            return Ok(());
        };
        if *installed || follows_transition.0 != transition as u64 {
            return Ok(());
        }
        let state = self.current.get(&&initial.main).ok_or(Refusal::Malformed)?;
        if position != 0 || state.removed || !state.matches_empty(before) {
            return Err(Refusal::Malformed);
        }
        self.record(&initial.main, Axes::empty(after), request)?;
        *installed = true;
        Ok(())
    }
    fn transition(
        &mut self,
        main: &BranchId,
        entry: &'a TransitionOriginalRecord,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        if let Transition::Fork {
            target,
            source,
            source_generation,
            source_truth_version,
            source_head,
        } = &entry.transition
        {
            let mut state = self
                .current
                .get(&source)
                .cloned()
                .ok_or(Refusal::Malformed)?;
            if self.current.get(&target).is_some()
                || state.removed
                || state.lifecycle != Lifecycle::Live
                || state.generation != *source_generation
                || state.truth != *source_truth_version
                || !state.matches_basis(source_head)
                || state.schema != entry.schema
            {
                return Err(Refusal::Malformed);
            }
            state.generation = worth_foundational::FoundationalBranchReferenceGeneration::initial();
            state.truth = crate::branch::RelationalBranchVersion::initial();
            return self.record(target, state, request);
        }
        let branch = entry.transition.branch();
        let mut state = self
            .current
            .get(&branch)
            .cloned()
            .ok_or(Refusal::Malformed)?;
        let (generation, truth) = match &entry.transition {
            Transition::Archive {
                prior_generation,
                prior_truth_version,
                ..
            }
            | Transition::DeletionRequested {
                prior_generation,
                prior_truth_version,
                ..
            }
            | Transition::Delete {
                prior_generation,
                prior_truth_version,
                ..
            } => (*prior_generation, *prior_truth_version),
            Transition::Fork { .. } => unreachable!("fork is handled above"),
        };
        if state.removed
            || state.generation != generation
            || state.truth != truth
            || state.schema != entry.schema
        {
            return Err(Refusal::Malformed);
        }
        match &entry.transition {
            Transition::Archive { .. } => {
                if state.lifecycle != Lifecycle::Live {
                    return Err(Refusal::Malformed);
                }
                state.generation = state
                    .generation
                    .checked_advance()
                    .map_err(|_| Refusal::Malformed)?;
                state.lifecycle = Lifecycle::Archived;
            }
            Transition::DeletionRequested { .. } => {
                if branch == main {
                    return Err(Refusal::Malformed);
                }
                state.lifecycle = Lifecycle::Deleting;
            }
            Transition::Delete { head_commit, .. } => {
                if branch == main
                    || state.commit.map(|index| index.commit.commit_id) != *head_commit
                {
                    return Err(Refusal::Malformed);
                }
                state.lifecycle = Lifecycle::Deleting;
                state.removed = true;
            }
            Transition::Fork { .. } => unreachable!("fork is handled above"),
        }
        self.record(branch, state, request)
    }
    fn require_component(
        &self,
        descriptor: &crate::branch::RelationalBranchBasisDescriptor,
        schema: [u8; 32],
    ) -> Result<(), Refusal> {
        let posture = match descriptor.posture() {
            crate::branch::RelationalBranchBasisPosture::Live => 0,
            crate::branch::RelationalBranchBasisPosture::Archived => 1,
            crate::branch::RelationalBranchBasisPosture::Deleting => 2,
        };
        let state = self
            .observations
            .get(&(
                descriptor.branch_id(),
                descriptor.reference().generation().get(),
                posture,
            ))
            .ok_or(Refusal::Malformed)?;
        if state.truth != descriptor.truth_version()
            || state.schema != schema
            || !state.matches_target(descriptor.reference().target())
        {
            return Err(Refusal::Malformed);
        }
        Ok(())
    }
}
fn map(cause: worth_execution::ExecutionOrderedMapDenial) -> Refusal {
    Refusal::Native(crate::durability::checkpoint_history::capture::map_stop(
        cause,
    ))
}
