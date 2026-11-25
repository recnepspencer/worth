//! Horizon readmission when older commits were durably disposed before reopen.
use super::{Descriptors, Points, ReferenceKey};
use crate::branch::ProductBranchObservation;
use crate::durability::{
    WorldNativeReadmissionCustody, WorldOpenAttemptRecord, WorldReadmissionDenial as Denial,
    WorldRecord, WorldRecoveredJournalState,
};
use crate::lifecycle::owner::{
    ReadmittedHorizonCommitSpec, ReadmittedHorizonInstall, RuntimeWorldOwnerRoot,
};
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{DurableCompositeCommitIdentity, DurableProductBranchIncarnation};
use worth_relational::facade::{
    branch::RecordedRelationalComponentReference, durability::RecoveredRelationalRuntimeAuthority,
    runtime::RelationalRuntime,
};
use worth_runtime_bridge::facade::RecordedRuntimeWorldBaseline;
use worth_signal::facade::durability::SignalRecordedSourcePoint;

mod created;
#[cfg(test)]
pub(crate) use created::require_retained_native_predecessor;

#[derive(Default)]
pub(super) struct HorizonContext {
    pub(super) root_incarnation: Option<DurableProductBranchIncarnation>,
    pub(super) bridges: BTreeMap<DurableProductBranchIncarnation, RecordedRuntimeWorldBaseline>,
    pub(super) native_branches:
        BTreeMap<DurableProductBranchIncarnation, worth_relational::facade::history::BranchId>,
    pub(super) disposed_native_commits: BTreeMap<
        DurableCompositeCommitIdentity,
        (worth_relational::facade::history::BranchId, u64),
    >,
    pub(super) creation_boundaries:
        BTreeMap<DurableProductBranchIncarnation, DurableCompositeCommitIdentity>,
    /// Maps each commit (surviving or disposed) to its nearest surviving
    /// ancestor observation and the number of original generations between them.
    pub(super) ancestors:
        BTreeMap<DurableCompositeCommitIdentity, (ProductBranchObservation, usize)>,
    /// Latest installed observation for each branch incarnation.
    pub(super) branch_heads: BTreeMap<DurableProductBranchIncarnation, ProductBranchObservation>,
}

impl HorizonContext {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn note_disposed<D, I, E, Ctx, T>(
        &mut self,
        root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
        key: &ReferenceKey,
        record: &WorldRecord,
        event_position: u64,
        recovered: &WorldRecoveredJournalState,
        native: &RelationalRuntime,
        authority: &RecoveredRelationalRuntimeAuthority,
        effects: &super::super::native_effects::ReadmittedNativeEffects,
        descriptors: &mut Descriptors,
        points: &mut Points,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial>
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
    {
        match record {
            WorldRecord::Branch {
                head, components, ..
            } => {
                self.root_incarnation = Some(head.incarnation);
                self.bridges.insert(key.0, components.bridge.clone());
                self.native_branches.insert(
                    key.0,
                    components.relational.descriptor().branch_id().clone(),
                );
                super::insert(descriptors, &head.commit, &components.relational, execution)?;
                super::insert(points, key, components.signal.point(), execution)?;
            }
            WorldRecord::BranchCreated(created) => {
                self.bridges
                    .insert(key.0, created.components.bridge.clone());
                self.native_branches.insert(
                    key.0,
                    created
                        .components
                        .relational
                        .descriptor()
                        .branch_id()
                        .clone(),
                );
                self.creation_boundaries
                    .insert(key.0, created.intent.source.commit);
                if created.intent.destination.commit != created.intent.source.commit
                    && let Some((ancestor, dist)) =
                        self.ancestors.get(&created.intent.source.commit).cloned()
                {
                    self.ancestors.insert(
                        created.intent.destination.commit,
                        (ancestor, dist.saturating_add(1)),
                    );
                }
                super::insert(
                    descriptors,
                    &created.intent.destination.commit,
                    &created.components.relational,
                    execution,
                )?;
                super::insert(points, key, created.components.signal.point(), execution)?;
            }
            WorldRecord::OpenAttempt(open) => {
                let occurrence = recovered
                    .occurrences
                    .get(&event_position)
                    .ok_or(Denial::MissingPredecessor)?;
                if let Some((ancestor, dist)) = self.ancestors.get(&open.expected.commit).cloned() {
                    self.ancestors
                        .insert(open.commit, (ancestor, dist.saturating_add(1)));
                }
                if let Some(recorded) = occurrence.relational() {
                    if let Some(branch_id) = self.native_branches.get(&key.0).cloned() {
                        self.disposed_native_commits
                            .insert(open.commit, (branch_id, recorded.commit));
                    }
                    if let Some(effect) = effects.take(&open.association.attempt())
                        && let Ok(after) =
                            authority.recorded_publication_successor(execution, native, &effect)
                    {
                        super::insert(descriptors, &open.commit, &after, execution)?;
                    }
                } else {
                    if let Some(before) = descriptors.get(&open.expected.commit).cloned() {
                        super::insert(descriptors, &open.commit, &before, execution)?;
                    }
                    if let Some(prev) = self
                        .disposed_native_commits
                        .get(&open.expected.commit)
                        .cloned()
                    {
                        self.disposed_native_commits.insert(open.commit, prev);
                    }
                }
                let point = match occurrence.owners().signal() {
                    Some(actual) => actual.point().clone(),
                    None => points
                        .get(&(open.expected.incarnation, open.expected.generation))
                        .ok_or(Denial::MissingPredecessor)?
                        .clone(),
                };
                super::insert(points, key, &point, execution)?;
            }
            _ => return Err(Denial::BranchLineage),
        }
        // If an earlier generation of a retired branch was installed (e.g.
        // retained for a surviving child) while its retired head was disposed,
        // retire the branch cell here so it does not stay live or block name reuse.
        if let Some(retired) = recovered.retired_branches.get(&key.0)
            && retired.head().generation == key.1
            && let Some(lifecycle) =
                crate::identity::readmission::readmit_incarnation(root.owner_identity(), key.0)
            && let Ok((cell, retirement)) = root
                .state
                .branches
                .retire_readmitted_by_lifecycle(lifecycle)
        {
            drop(cell);
            drop(retirement);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn install_surviving_open<D, I, E, Ctx, T>(
        &self,
        root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
        key: &ReferenceKey,
        open: &WorldOpenAttemptRecord,
        event_position: u64,
        recovered: &WorldRecoveredJournalState,
        descriptors: &Descriptors,
        points: &Points,
        native: &RelationalRuntime,
        authority: &RecoveredRelationalRuntimeAuthority,
        effects: &super::super::native_effects::ReadmittedNativeEffects,
        custody: &mut WorldNativeReadmissionCustody,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            ProductBranchObservation,
            RecordedRelationalComponentReference,
            SignalRecordedSourcePoint,
        ),
        Denial,
    >
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
    {
        let occurrence = recovered
            .occurrences
            .get(&event_position)
            .ok_or(Denial::MissingPredecessor)?;
        let (native_commit, after) = match occurrence.relational() {
            Some(recorded) => {
                let effect = effects
                    .take(&open.association.attempt())
                    .ok_or(Denial::OccurrenceEvidenceMismatch)?;
                let subject = effect.subject();
                if subject.receipt().commit_id.0 != recorded.commit
                    || subject.position().0 != recorded.position
                    || subject.artifact_digest() != &recorded.digest
                {
                    return Err(Denial::OccurrenceEvidenceMismatch);
                }
                let commit_identity = native
                    .history()
                    .immutable_commit_identity(subject.receipt().commit_id)
                    .ok_or(Denial::OccurrenceEvidenceMismatch)?;
                let after = authority
                    .recorded_publication_successor(execution, native, &effect)
                    .map_err(Denial::NativeSuccessor)?;
                if let Some(before_native) = descriptors.get(&open.expected.commit) {
                    authority
                        .validate_recorded_publication_predecessor(
                            execution,
                            native,
                            &effect,
                            before_native,
                            &after,
                        )
                        .map_err(|_| Denial::OccurrenceEvidenceMismatch)?;
                } else if let Some((expected_branch, expected_commit)) =
                    self.disposed_native_commits.get(&open.expected.commit)
                {
                    authority
                        .validate_recorded_publication_pruned_predecessor(
                            execution,
                            native,
                            &effect,
                            expected_branch,
                            worth_relational::facade::history::CommitId(*expected_commit),
                            &after,
                        )
                        .map_err(|_| Denial::OccurrenceEvidenceMismatch)?;
                } else {
                    return Err(Denial::MissingPredecessor);
                }
                (Some(commit_identity), after)
            }
            None => {
                if effects.take(&open.association.attempt()).is_some() {
                    return Err(Denial::OccurrenceEvidenceMismatch);
                }
                (
                    None,
                    descriptors
                        .get(&open.expected.commit)
                        .ok_or(Denial::MissingPredecessor)?
                        .clone(),
                )
            }
        };
        let signal_port = root
            .state
            .signal
            .authored()
            .ok_or(Denial::AuthoredSignalRequired)?
            .source_port();
        let (signal_changed, point, admitted_signal) = match occurrence.owners().signal() {
            Some(actual) => {
                let admitted = signal_port
                    .readmit_source_point(actual.point(), execution)
                    .map_err(|cause| Denial::SignalSource(Box::new(cause)))?;
                if admitted.original().association() != Some(&open.association) {
                    return Err(Denial::OccurrenceEvidenceMismatch);
                }
                (true, actual.point().clone(), admitted)
            }
            None => {
                let point = points
                    .get(&(open.expected.incarnation, open.expected.generation))
                    .ok_or(Denial::MissingPredecessor)?
                    .clone();
                let admitted = signal_port
                    .readmit_source_point(&point, execution)
                    .map_err(|cause| Denial::SignalSource(Box::new(cause)))?;
                (false, point, admitted)
            }
        };
        let native_basis = authority
            .admit_recorded_component(execution, native, &after)
            .map_err(|cause| Denial::NativeComponent(Box::new(cause)))?;
        let index = custody.roots.len();
        custody.roots.push(Some(native_basis));
        let native_basis = custody.roots[index]
            .take()
            .expect("actual acquired Native horizon basis")
            .into_basis();
        let bridge_baseline = self.bridges.get(&key.0).ok_or(Denial::MissingPredecessor)?;
        let bridge = root
            .state
            .bridge
            .as_ref()
            .expect("installed correspondence")
            .readmit_recorded_runtime_world_baseline(bridge_baseline, admitted_signal.basis())
            .map_err(Denial::BridgeBaseline)?;
        let spliced_parent = self
            .ancestors
            .get(&open.expected.commit)
            .map(|(obs, dist)| (obs.selected_commit().clone(), dist.saturating_add(1)));
        let (observed, origin) = root.install_readmitted_horizon_commit(
            ReadmittedHorizonInstall {
                branch_name: open.expected.branch.name(),
                incarnation: key.0,
                generation: key.1,
                commit: open.commit,
                is_root_branch: self.root_incarnation == Some(key.0),
                retirement_boundary: self.creation_boundaries.get(&key.0).copied(),
                branch_previous: self.branch_heads.get(&key.0).map(|o| o.snapshot()),
                spliced_parent,
                commit_spec: ReadmittedHorizonCommitSpec::Publication {
                    original_parent: open.expected.commit,
                    attempt: open.association.attempt(),
                    native_commit,
                    signal_change: signal_changed.then_some(&admitted_signal),
                },
                native: native_basis,
                signal: &admitted_signal,
                bridge,
                expected_native: &after,
                expected_signal: &point,
            },
            execution,
        )?;
        custody.checkpoint_occurrences.push(origin);
        Ok((observed, after, point))
    }
}
