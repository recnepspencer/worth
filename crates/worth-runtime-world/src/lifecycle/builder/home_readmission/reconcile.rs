//! Reconstruct exact references in structural order while retaining every acquired owner.
use crate::durability::{
    WorldDurableMediumBinding, WorldNativeReadmissionCustody, WorldReadmissionDenial as Denial,
    WorldRecord, WorldRecoveredJournalState,
};
use crate::lifecycle::owner::RuntimeWorldOwnerRoot;
use std::sync::Arc;
use worth_execution::ExecutionOrderedMap;
use worth_foundational::facade::{
    DurableProductBranchIncarnation, DurableProductReferenceGeneration,
};
use worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority;
mod creation;
mod horizon;
mod unpublished;
mod unresolved_creation;

type ReferenceKey = (
    DurableProductBranchIncarnation,
    DurableProductReferenceGeneration,
);
type Observations = ExecutionOrderedMap<ReferenceKey, crate::branch::ProductBranchObservation>;
type Points =
    ExecutionOrderedMap<ReferenceKey, worth_signal::facade::durability::SignalRecordedSourcePoint>;
type Descriptors = ExecutionOrderedMap<
    worth_foundational::facade::DurableCompositeCommitIdentity,
    worth_relational::facade::branch::RecordedRelationalComponentReference,
>;

pub(super) fn reconcile<D, I, E, Ctx, T>(
    root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    binding: &Arc<WorldDurableMediumBinding>,
    recovered: &WorldRecoveredJournalState,
    native: &worth_relational::facade::runtime::RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    effects: super::native_effects::ReadmittedNativeEffects,
    signal_effects: super::signal_effects::ReadmittedSignalEffects,
    creation_forks: super::creation_effects::UnresolvedCreationForks,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let mut custody = WorldNativeReadmissionCustody::allocate(
        execution,
        recovered.branches.len() + recovered.occurrences.len(),
        recovered.occurrences.len(),
        recovered.opens.len(),
        recovered.creations.len(),
    )?;
    custody.history = Some(root.state.history.clone());
    custody.branches = Some(root.state.branches.clone());
    custody.recovery = Some(root.state.recovery.clone());
    let result = (|| {
        let signal = root
            .state
            .signal
            .authored()
            .ok_or(Denial::AuthoredSignalRequired)?;
        let mut observations = Observations::default();
        let mut points = Points::default();
        let mut descriptors = Descriptors::default();
        let mut horizon = horizon::HorizonContext::default();
        for (key, reference) in &recovered.references {
            execution.consult().map_err(|cause| {
                Denial::Medium(crate::durability::WorldDurableMediumDenial::Request(cause))
            })?;
            let commit_id = reference.fields().2;
            if recovered.retired_history.contains_key(&commit_id) {
                horizon.note_disposed(
                    root,
                    key,
                    &reference.record().record,
                    reference.event_position(),
                    recovered,
                    native,
                    authority,
                    &effects,
                    &mut descriptors,
                    &mut points,
                    execution,
                )?;
                continue;
            }
            let (observation, after, point) = match &reference.record().record {
                WorldRecord::Branch {
                    head,
                    components,
                    ..
                } => {
                    horizon.root_incarnation = Some(head.incarnation);
                    horizon.bridges.insert(key.0, components.bridge.clone());
                    let native_basis = authority
                        .admit_recorded_component(execution, native, &components.relational)
                        .map_err(|cause| Denial::NativeComponent(Box::new(cause)))?;
                    let index = custody.roots.len();
                    custody.roots.push(Some(native_basis));
                    let admitted_signal = signal
                        .source_port()
                        .readmit_source_point(components.signal.point(), execution)
                        .map_err(|cause| Denial::SignalSource(Box::new(cause)))?;
                    let bridge = root
                        .state
                        .bridge
                        .as_ref()
                        .expect("installed correspondence")
                        .readmit_recorded_runtime_world_baseline(
                            &components.bridge,
                            admitted_signal.basis(),
                        )
                        .map_err(Denial::BridgeBaseline)?;
                    let (observed, origin) = root.install_readmitted_root(
                        reference.record(),
                        custody.roots[index]
                            .take()
                            .expect("actual acquired Native root"),
                        admitted_signal,
                        bridge,
                        execution,
                    )?;
                    custody.checkpoint_origins.push(origin);
                    (
                        observed,
                        components.relational.clone(),
                        components.signal.point().clone(),
                    )
                }
                WorldRecord::BranchCreated(created) => {
                    horizon.bridges.insert(key.0, created.components.bridge.clone());
                    horizon
                        .creation_boundaries
                        .insert(key.0, created.intent.source.commit);
                    if let Ok(before) = find(&observations, &created.intent.source) {
                        let before_native = descriptors
                            .get(&created.intent.source.commit)
                            .ok_or(Denial::MissingPredecessor)?;
                        let source_point = points
                            .get(&(
                                created.intent.source.incarnation,
                                created.intent.source.generation,
                            ))
                            .ok_or(Denial::MissingPredecessor)?;
                        let observed = creation::install(
                            root,
                            created,
                            before,
                            before_native,
                            source_point,
                            native,
                            authority,
                            &mut custody,
                            execution,
                        )?;
                        (
                            observed,
                            created.components.relational.clone(),
                            created.components.signal.point().clone(),
                        )
                    } else {
                        let before_native = descriptors
                            .get(&created.intent.source.commit)
                            .ok_or(Denial::MissingPredecessor)?;
                        let source_point = points
                            .get(&(
                                created.intent.source.incarnation,
                                created.intent.source.generation,
                            ))
                            .ok_or(Denial::MissingPredecessor)?;
                        horizon.install_surviving_created(
                            root,
                            created,
                            before_native,
                            source_point,
                            native,
                            authority,
                            &mut custody,
                            execution,
                        )?
                    }
                }
                WorldRecord::OpenAttempt(open) => {
                    if let Ok(before) = find(&observations, &open.expected) {
                        let occurrence = recovered
                            .occurrences
                            .get(&reference.event_position())
                            .ok_or(Denial::MissingPredecessor)?;
                        let before_native = descriptors
                            .get(&open.expected.commit)
                            .ok_or(Denial::MissingPredecessor)?;
                        let (owners, after) = super::occurrence_owners::readmit(
                            occurrence,
                            before_native,
                            signal,
                            native,
                            authority,
                            &effects,
                            execution,
                        )?;
                        let index = custody.occurrence_owners.len();
                        custody.occurrence_owners.push(Some(owners));
                        let mut origin =
                            crate::durability::PreparedCheckpointOccurrence::prepare(execution)
                                .map_err(Denial::Medium)?;
                        let record = match root.prepare_readmitted_occurrence(
                            execution,
                            before,
                            occurrence,
                            custody.occurrence_owners[index]
                                .take()
                                .expect("same actual owners"),
                            before.basis().correspondence_basis().clone(),
                        ) {
                            Ok(value) => value,
                            Err((owners, cause)) => {
                                custody.occurrence_owners[index] = Some(owners);
                                return Err(cause);
                            }
                        };
                        custody.records.push(record.clone());
                        let selected =
                            root.install_readmitted_occurrence(&record, &mut custody.capacities)?;
                        origin.bind(
                            selected.selected_commit().ordinal(),
                            selected.basis(),
                            selected
                                .fork_checkpoint_history()
                                .map_err(|_| Denial::RecoveryCapacity)?,
                        );
                        custody.checkpoint_occurrences.push(origin);
                        let point = match occurrence.owners().signal() {
                            Some(actual) => actual.point().clone(),
                            None => points
                                .get(&(open.expected.incarnation, open.expected.generation))
                                .ok_or(Denial::MissingPredecessor)?
                                .clone(),
                        };
                        (selected, after, point)
                    } else {
                        horizon.install_surviving_open(
                            root,
                            key,
                            open,
                            reference.event_position(),
                            recovered,
                            &descriptors,
                            &points,
                            native,
                            authority,
                            &effects,
                            &mut custody,
                            execution,
                        )?
                    }
                }
                WorldRecord::Owner { .. }
                | WorldRecord::Occurrence { .. }
                | WorldRecord::Released { .. }
                | WorldRecord::Continued { .. }
                | WorldRecord::PlannedSignalLeg(_)
                | WorldRecord::CreationIntent(_)
                | WorldRecord::CreationReleased { .. }
                | WorldRecord::BranchRetired { .. }
                | WorldRecord::HistoryRetired { .. } => return Err(Denial::BranchLineage),
            };
            horizon.ancestors.insert(commit_id, (observation.clone(), 0));
            horizon.branch_heads.insert(key.0, observation.clone());
            custody.observations.push(observation.clone());
            insert(&mut observations, key, &observation, execution)?;
            insert(&mut descriptors, &commit_id, &after, execution)?;
            insert(&mut points, key, &point, execution)?;
            if let Some(retired) = recovered.retired_branches.get(&key.0)
                && retired.head().generation == key.1
            {
                let (cell, retirement) = root
                    .state
                    .branches
                    .retire_readmitted_by_lifecycle(observation.lifecycle_incarnation())
                    .map_err(|_| Denial::BranchLineage)?;
                drop(cell);
                drop(retirement);
            }
        }
        unpublished::install(
            root,
            binding,
            recovered,
            native,
            authority,
            &effects,
            &signal_effects,
            &descriptors,
            &observations,
            &mut custody,
            execution,
        )?;
        unresolved_creation::install(
            root,
            binding,
            recovered,
            native,
            authority,
            &unresolved_creation::OwnerInventories {
                forks: &creation_forks,
                signal: &signal_effects,
            },
            &unresolved_creation::RecoveredReferences {
                observations: &observations,
                descriptors: &descriptors,
                points: &points,
            },
            &mut custody,
            execution,
        )?;
        creation::retire_admissions(&mut custody, native, execution)?;
        // All fallible readmission steps have succeeded.
        for origin in custody.checkpoint_origins.drain(..) {
            binding.retain_checkpoint_origin(origin);
        }
        for origin in custody.checkpoint_occurrences.drain(..) {
            binding.retain_checkpoint_origin(origin);
        }
        binding.mark_checkpoint_origin_installed();
        Ok(())
    })();
    result.map_err(|cause| custody.refuse(cause))
}

fn find<'a>(
    map: &'a Observations,
    head: &crate::durability::DurableWorldHead,
) -> Result<&'a crate::branch::ProductBranchObservation, Denial> {
    let observed = map
        .get(&(head.incarnation, head.generation))
        .ok_or(Denial::MissingPredecessor)?;
    if observed.owner_identity().durable() != Some(head.branch.owner())
        || observed.branch_identity().name().as_str() != head.branch.name()
        || observed.selected_commit().ordinal() != head.commit.ordinal()
    {
        return Err(Denial::BranchLineage);
    }
    Ok(observed)
}

fn insert<K: Ord + Clone, V: Clone>(
    map: &mut ExecutionOrderedMap<K, V>,
    key: &K,
    value: &V,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    map.insert_cloned_native_for_request(key, value, *request)
        .map_err(|cause| {
            Denial::Medium(crate::durability::WorldDurableMediumDenial::Ordered(cause))
        })
}
