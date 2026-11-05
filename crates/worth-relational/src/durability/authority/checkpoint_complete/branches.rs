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
use axes::Axes;

pub(super) struct BranchImages<'a> {
    current: ExecutionOrderedMap<&'a BranchId, Axes>,
    observations: ExecutionOrderedMap<(&'a BranchId, u64, u8), Axes>,
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
            let checkpoint = commit
                .envelope
                .branch_cell_checkpoint
                .as_ref()
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            let branch = &commit.envelope.branch_context;
            let mut before = image
                .current
                .get(&branch)
                .cloned()
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            if before.removed
                || before.lifecycle != Lifecycle::Live
                || checkpoint.branch_id != *branch
                || !before.matches(
                    checkpoint.observation.generation(),
                    checkpoint.truth_version,
                    checkpoint.observation.target(),
                    roots,
                )
                || !Arc::ptr_eq(&before.schema, &commit.inputs.prestate_schema)
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
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
            image.record(branch, before, execution)?;
        }
        if transition != transitions.len() || original.installation.is_some() != installation_done {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        Ok(image)
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

    fn transition(
        &mut self,
        main: &BranchId,
        entry: &'a RetainedBranchTransition,
        roots: &FoldedRoots,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let record = entry.positioned.transition();
        if matches!(record, RelationalBranchTransition::RetainedFork { .. }) {
            return Err(Denial::Durability(crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::UnsupportedRetainedForkReplay,
                "retained-source forks require original live root custody; historical tail reconstruction is unsupported",
            )));
        }

        if let RelationalBranchTransition::Fork {
            target,
            source,
            source_generation,
            source_truth_version,
            source_head,
        } = record
        {
            let prior = self
                .current
                .get(&source)
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            if self.current.get(&target).is_some()
                || prior.removed
                || prior.lifecycle != Lifecycle::Live
                || (prior.generation != *source_generation
                    || prior.truth != *source_truth_version
                    || !prior.matches_basis(source_head, roots))
                || !Arc::ptr_eq(&prior.schema, &entry.schema)
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            let mut next = prior.clone();
            next.generation = Generation::initial();
            next.truth = RelationalBranchVersion::initial();
            self.record(target, next, execution)?;
            return Ok(());
        }
        let branch = record.branch();
        let mut prior = self
            .current
            .get(&branch)
            .cloned()
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let (generation, truth) = match record {
            RelationalBranchTransition::Archive {
                prior_generation,
                prior_truth_version,
                ..
            }
            | RelationalBranchTransition::DeletionRequested {
                prior_generation,
                prior_truth_version,
                ..
            }
            | RelationalBranchTransition::Delete {
                prior_generation,
                prior_truth_version,
                ..
            } => (*prior_generation, *prior_truth_version),
            RelationalBranchTransition::Fork { .. }
            | RelationalBranchTransition::RetainedFork { .. } => {
                unreachable!("fork is handled above")
            }
        };
        if prior.removed
            || prior.generation != generation
            || prior.truth != truth
            || !Arc::ptr_eq(&prior.schema, &entry.schema)
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        match record {
            RelationalBranchTransition::Archive { .. } => {
                if prior.lifecycle != Lifecycle::Live {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                prior.generation = prior
                    .generation
                    .checked_advance()
                    .map_err(|_| Denial::OriginalAuthorityUnavailable)?;
                prior.lifecycle = Lifecycle::Archived;
            }
            RelationalBranchTransition::DeletionRequested { .. } => {
                if branch == main {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                prior.lifecycle = Lifecycle::Deleting;
            }
            RelationalBranchTransition::Delete { head_commit, .. } => {
                if branch == main || prior.commit != *head_commit {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                prior.lifecycle = Lifecycle::Deleting;
                prior.removed = true;
            }
            RelationalBranchTransition::Fork { .. }
            | RelationalBranchTransition::RetainedFork { .. } => {
                unreachable!("fork is handled above")
            }
        }
        self.record(branch, prior, execution)
    }

    fn record(
        &mut self,
        branch: &'a BranchId,
        axes: Axes,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let key = (branch, axes.generation.get(), posture(axes.lifecycle));
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
