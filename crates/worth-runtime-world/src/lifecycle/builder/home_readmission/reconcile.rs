//! Private staged history owns every acquired Native admission on refusal.
use crate::durability::{
    RecoveredCustodyDisposition, WorldDurableMediumBinding, WorldNativeReadmissionCustody,
    WorldReadmissionDenial as Denial, WorldRecord, WorldRecoveredJournalState,
};
use crate::identity::readmission::{readmit_attempt, readmit_commit, readmit_custody};
use crate::lifecycle::owner::RuntimeWorldOwnerRoot;
use crate::recovery::RecoveredWorldPublicationInputs;
use std::sync::Arc;
use worth_execution::ExecutionOrderedMap;
use worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority;

pub(super) fn reconcile<D, I, E, Ctx, T>(
    root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    binding: &Arc<WorldDurableMediumBinding>,
    recovered: &WorldRecoveredJournalState,
    native: &worth_relational::facade::runtime::RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    effects: super::native_effects::ReadmittedNativeEffects,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let Some(first) = recovered
        .branch_origins()
        .min_by_key(|origin| origin.position)
    else {
        return Ok(());
    };
    let origins = || {
        std::iter::once(first).chain(
            recovered
                .branch_origins()
                .filter(|origin| origin.position != first.position),
        )
    };
    let owner = root.owner_identity();
    let mut custody = WorldNativeReadmissionCustody::allocate(
        execution,
        recovered.branches.len(),
        recovered.occurrences.len(),
        recovered.opens.len(),
    )?;
    // These are the SAME private catalog/registry/recovery allocations. A late
    // refusal retains any already materialized root and history, not just IDs.
    custody.history = Some(root.state.history.clone());
    custody.branches = Some(root.state.branches.clone());
    custody.recovery = Some(root.state.recovery.clone());
    let result = (|| {
        let signal = root
            .state
            .signal
            .authored()
            .ok_or(Denial::AuthoredSignalRequired)?;
        let mut descriptors = ExecutionOrderedMap::default();
        for origin in origins() {
            let WorldRecord::Branch {
                components, head, ..
            } = &origin.record().record
            else {
                unreachable!("branch source")
            };
            let native_basis = authority
                .admit_recorded_component(execution, native, &components.relational)
                .map_err(|cause| Denial::NativeComponent(Box::new(cause)))?;
            custody.roots.push(Some(native_basis));
            insert(
                &mut descriptors,
                &head.commit,
                &components.relational,
                execution,
            )?;
        }
        let mut published = ExecutionOrderedMap::default();
        for occurrence in recovered.occurrences.values() {
            let open = occurrence.open();
            let before = descriptors
                .get(&open.expected.commit)
                .ok_or(Denial::MissingPredecessor)?;
            let (admitted, after) = super::occurrence_owners::readmit(
                occurrence, before, signal, native, authority, &effects, execution,
            )?;
            custody.occurrence_owners.push(Some(admitted));
            insert(&mut descriptors, &open.commit, &after, execution)?;
            insert(&mut published, &open.association.attempt(), &(), execution)?;
        }
        let mut opens = worth_execution::ExecutionOrderedMap::default();
        for open in recovered.opens.values() {
            if published.contains_key(&open.association.attempt())
                || matches!(
                    recovered
                        .dispositions
                        .get(&open.association.attempt())
                        .map(|row| row.disposition()),
                    Some(RecoveredCustodyDisposition::Released(_))
                )
            {
                continue;
            }
            let before = descriptors
                .get(&open.expected.commit)
                .ok_or(Denial::MissingPredecessor)?;
            let admitted = if let Some(effect) = effects.take(&open.association.attempt()) {
                let after = authority
                    .recorded_publication_successor(execution, native, &effect)
                    .map_err(Denial::NativeSuccessor)?;
                Some(
                    authority
                        .admit_recorded_publication(execution, native, effect, before, &after)
                        .map_err(|cause| Denial::NativePublication(Box::new(cause)))?,
                )
            } else {
                None
            };
            if admitted.is_none()
                && matches!(
                    recovered
                        .dispositions
                        .get(&open.association.attempt())
                        .map(|row| row.disposition()),
                    Some(RecoveredCustodyDisposition::Continued)
                )
            {
                return Err(Denial::OccurrenceEvidenceMismatch);
            }
            let index = custody.publications.len();
            custody.publications.push(admitted);
            let ids = (
                readmit_attempt(owner, open.association.attempt()).ok_or(Denial::ForeignOwner)?,
                readmit_commit(owner, open.commit).ok_or(Denial::ForeignOwner)?,
                readmit_custody(owner, open.custody).ok_or(Denial::ForeignOwner)?,
            );
            let slot = root
                .state
                .recovery
                .reserve_product_unpublished(owner)
                .map_err(|_| Denial::RecoveryCapacity)?;
            let slot_index = custody.slots.len();
            custody.slots.push(Some(slot));
            insert(
                &mut opens,
                &open.association.attempt(),
                &(open.clone(), ids, index, slot_index),
                execution,
            )?;
        }
        let mut observations = ExecutionOrderedMap::default();
        for (index, origin) in origins().enumerate() {
            let WorldRecord::Branch {
                components, head, ..
            } = &origin.record().record
            else {
                unreachable!("branch source")
            };
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
            let native = custody.roots[index]
                .take()
                .expect("same admitted Native branch");
            let (expected, origin) = root.install_readmitted_root(
                origin.record(),
                native,
                admitted_signal,
                bridge,
                execution,
            )?;
            custody.checkpoint_origins.push(origin);
            custody.observations.push(expected.clone());
            insert(&mut observations, &head.commit, &expected, execution)?;
        }
        for (index, occurrence) in recovered.occurrences.values().enumerate() {
            let open = occurrence.open();
            let before = observations
                .get(&open.expected.commit)
                .ok_or(Denial::MissingPredecessor)?;
            let mut origin = crate::durability::PreparedCheckpointOccurrence::prepare(execution)
                .map_err(Denial::Medium)?;
            let admitted = custody.occurrence_owners[index]
                .take()
                .expect("same acquired occurrence cap");
            let record = match root.prepare_readmitted_occurrence(
                execution,
                before,
                occurrence,
                admitted,
                before.basis().correspondence_basis().clone(),
            ) {
                Ok(record) => record,
                Err((admitted, cause)) => {
                    custody.occurrence_owners[index] = Some(admitted);
                    return Err(cause);
                }
            };
            custody.records.push(record.clone());
            let selected = root.install_readmitted_occurrence(&record, &mut custody.capacities)?;
            custody.observations.push(selected.clone());
            let pins = selected.fork_checkpoint_history().map_err(|_| {
                Denial::Medium(
                    crate::durability::WorldDurableMediumDenial::CheckpointOriginRetention,
                )
            })?;
            origin.bind(selected.basis(), pins);
            custody.checkpoint_occurrences.push(origin);
            insert(&mut observations, &open.commit, &selected, execution)?;
        }
        for disposition in recovered.dispositions.values() {
            if let RecoveredCustodyDisposition::Released(_) = disposition.disposition() {
                let open = disposition.open();
                let expected = observations
                    .get(&open.expected.commit)
                    .ok_or(Denial::MissingPredecessor)?;
                root.state
                    .recovery
                    .readmit_released_original(
                        execution,
                        disposition,
                        expected,
                        effects.take(&open.association.attempt()),
                    )
                    .map_err(Denial::Medium)?;
            }
        }
        for (open, (attempt, commit, identity), index, slot_index) in opens.values() {
            let expected = observations
                .get(&open.expected.commit)
                .ok_or(Denial::MissingPredecessor)?
                .clone();
            custody.slots[*slot_index]
                .take()
                .expect("same reserved slot")
                .install_recovered(RecoveredWorldPublicationInputs {
                    identity: identity.clone(),
                    attempt: attempt.clone(),
                    commit: commit.clone(),
                    continued: recovered
                        .dispositions
                        .get(&open.association.attempt())
                        .cloned(),
                    open: open.clone(),
                    expected,
                    binding: binding.clone(),
                    native_identity: custody.publications[*index]
                        .as_ref()
                        .map(|publication| {
                            native
                                .history()
                                .immutable_commit_identity(
                                    publication.custody().effect().subject().receipt().commit_id,
                                )
                                .ok_or(Denial::OccurrenceEvidenceMismatch)
                        })
                        .transpose()?,
                    native: custody.publications[*index].take(),
                });
        }
        // All fallible reconciliation is complete. Until this handoff the
        // owning refusal retains every recipe with its private World graph;
        // the Home binding must not outlive cleanup with a partial recipe.
        for origin in custody.checkpoint_origins.drain(..) {
            binding.retain_checkpoint_origin(origin);
        }
        for origin in custody.checkpoint_occurrences.drain(..) {
            binding.retain_checkpoint_occurrence(origin);
        }
        Ok(())
    })();
    result.map_err(|cause| custody.refuse(cause))
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
