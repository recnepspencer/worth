//! Runtime-neutral DATA validation of every observed branch component.
use super::{roots::FoldedRoots, source::OriginalCanonicalCommit};
use crate::branch::{
    RelationalBranchBasisDescriptor, RelationalBranchBasisPosture,
    RelationalBranchLifecyclePosture as Lifecycle, RelationalBranchRootSchemaAuthority,
    RelationalBranchTransition, RelationalBranchVersion,
};
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use crate::history::data::{BranchId, CommitId};
use crate::runtime::{
    OriginalBranchSchemaInputs, OriginalEmptySchemaObservation, RetainedBranchTransition,
};
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_foundational::{
    FoundationalBranchReferenceGeneration as Generation, FoundationalBranchTarget,
};

mod axes;
mod component_admission;
mod transitions;
mod world_releases;
use axes::{Axes, ObservationKey, RequiredSource};
use world_releases::WorldReleases;

pub(super) struct BranchImages<'a> {
    current: ExecutionOrderedMap<&'a BranchId, Axes>,
    observations: ExecutionOrderedMap<ObservationKey<'a>, Axes>,
}

impl<'a> BranchImages<'a> {
    pub(super) fn derive(
        main: &'a BranchId,
        original: &'a OriginalBranchSchemaInputs,
        commits: &'a [OriginalCanonicalCommit],
        roots: &FoldedRoots,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let birth = original
            .birth
            .as_ref()
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let mut image = Self {
            current: Default::default(),
            observations: Default::default(),
        };
        image.record(main, Axes::empty(birth), execution)?;
        let transitions = original.transitions();
        let mut releases = WorldReleases::prepare(original, execution)?;
        let mut transition = 0usize;
        let mut installation_done = false;
        for position in 0..=commits.len() {
            image.install_if_due(
                main,
                original,
                transition,
                &mut installation_done,
                execution,
            )?;
            while let Some(entry) = transitions.get(transition) {
                if entry.positioned.follows().0 != position as u64 {
                    break;
                }
                if entry.positioned.sequence().0 != transition as u64 + 1 {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                if !matches!(
                    entry.positioned.transition(),
                    RelationalBranchTransition::RetainedFork { .. }
                ) {
                    image.select_required_source(
                        RequiredSource::Transition(entry),
                        &releases,
                        roots,
                        execution,
                    )?;
                }
                image.transition(main, entry, roots, execution)?;
                transition += 1;
                image.install_if_due(
                    main,
                    original,
                    transition,
                    &mut installation_done,
                    execution,
                )?;
            }
            let Some(commit) = commits.get(position) else {
                break;
            };
            if commit.position.0 != position as u64 + 1 {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            let branch = &commit.envelope.branch_context;
            let mut before = image.select_required_source(
                RequiredSource::Canonical(commit),
                &releases,
                roots,
                execution,
            )?;
            let predecessor = before.clone();
            before.generation = before
                .generation
                .checked_advance()
                .map_err(|_| Denial::OriginalAuthorityUnavailable)?;
            before.truth = before
                .truth
                .checked_advance()
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            before.commit = Some(commit.envelope.commit.commit_id);
            before.schema = commit.inputs.result_schema.clone();
            releases.retain_original(
                commit,
                predecessor,
                before.clone(),
                commits,
                roots,
                execution,
            )?;
            image.record(branch, before, execution)?;
        }
        if transition != transitions.len() || original.installation.is_some() != installation_done {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        for restoration in releases.restorations() {
            if image
                .current
                .get(&restoration.branch)
                .is_some_and(|current| current.same_state(&restoration.after))
            {
                image.record(restoration.branch, restoration.before.clone(), execution)?;
            }
        }
        Ok(image)
    }

    fn select_required_source(
        &mut self,
        required: RequiredSource<'a>,
        releases: &WorldReleases<'a>,
        roots: &FoldedRoots,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Axes, Denial> {
        let branch = required.branch();
        let current = self
            .current
            .get(&branch)
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        if required.matches(current, roots) {
            return Ok(current.clone());
        }
        let predecessor = releases
            .predecessor(branch, current)
            .filter(|before| required.matches(before, roots))
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let selected = predecessor.clone();
        self.record(branch, selected.clone(), execution)?;
        Ok(selected)
    }

    fn install_if_due(
        &mut self,
        main: &'a BranchId,
        original: &'a OriginalBranchSchemaInputs,
        transition: usize,
        done: &mut bool,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let Some(installation) = &original.installation else {
            return Ok(());
        };
        if *done || installation.follows_transition.0 != transition as u64 {
            return Ok(());
        }
        let current = self
            .current
            .get(&main)
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        if current.commit.is_some()
            || current.removed
            || !current.is_empty_observation(&installation.before)
            || installation.after.truth_version != installation.before.truth_version
            || installation.after.lifecycle != installation.before.lifecycle
            || installation.after.generation
                != installation
                    .before
                    .generation
                    .checked_advance()
                    .map_err(|_| Denial::OriginalAuthorityUnavailable)?
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        self.record(main, Axes::empty(&installation.after), execution)?;
        *done = true;
        Ok(())
    }

    fn record(
        &mut self,
        branch: &'a BranchId,
        axes: Axes,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let key = ObservationKey::of_axes(branch, &axes);
        self.observations
            .insert_cloned_native_for_request(&key, &axes, *execution)
            .map_err(map_stop)?;
        self.current
            .insert_cloned_native_for_request(&branch, &axes, *execution)
            .map_err(map_stop)
    }

    pub(super) fn require_current(
        &self,
        cells: &[crate::branch::RelationalBranchCellCheckpoint],
        lifecycle: &[crate::durability::data::DurableBranchLifecycleImage],
        retired: &[BranchId],
        roots: &FoldedRoots,
    ) -> Result<(), Denial> {
        if cells.len()
            != self
                .current
                .iter()
                .filter(|(_, axes)| !axes.removed)
                .count()
            || lifecycle.len() != cells.len()
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        for (cell, lifecycle) in cells.iter().zip(lifecycle) {
            let state = self
                .current
                .get(&&cell.branch_id)
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            if state.removed
                || !state.matches(
                    cell.observation.generation(),
                    cell.truth_version,
                    cell.observation.target(),
                    roots,
                )
                || lifecycle.branch_id != cell.branch_id
                || lifecycle.posture != state.lifecycle
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
        }
        if retired.len()
            != self
                .current
                .iter()
                .filter(|(_, axes)| axes.lifecycle == Lifecycle::Deleting)
                .count()
            || retired.iter().any(|branch| {
                !self
                    .current
                    .get(&branch)
                    .is_some_and(|state| state.lifecycle == Lifecycle::Deleting)
            })
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        Ok(())
    }
}

fn posture(lifecycle: Lifecycle) -> u8 {
    match lifecycle {
        Lifecycle::Live => 0,
        Lifecycle::Archived => 1,
        Lifecycle::Deleting => 2,
    }
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    crate::durability::checkpoint_history::capture::capture_stop(
        crate::durability::checkpoint_history::capture::map_stop(cause),
    )
}
