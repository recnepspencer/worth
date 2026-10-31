//! Private staged history owns every acquired Native admission on refusal.
use crate::durability::{
    RecoveredCustodyDisposition, WorldDurableMediumBinding, WorldNativeReadmissionCustody,
    WorldReadmissionDenial as Denial, WorldRecord, WorldRecoveredJournalState,
};
use crate::identity::readmission::{readmit_attempt, readmit_commit, readmit_custody};
use crate::lifecycle::owner::RuntimeWorldOwnerRoot;
use crate::recovery::RecoveredWorldPublicationInputs;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity;
use worth_relational::facade::durability::{
    ReadmittedRelationalAssociatedEffect, RecoveredRelationalRuntimeAuthority,
};

pub(super) fn reconcile<D, I, E, Ctx, T>(
    root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    binding: &Arc<WorldDurableMediumBinding>,
    recovered: &WorldRecoveredJournalState,
    native: &worth_relational::facade::runtime::RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    mut effects: BTreeMap<
        DurableCompositePublicationAttemptIdentity,
        ReadmittedRelationalAssociatedEffect,
    >,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let Some(artifact) = recovered.branches.first() else {
        return Ok(());
    };
    let WorldRecord::Branch {
        components, head, ..
    } = &artifact.record
    else {
        unreachable!("branch frame")
    };
    let owner = root.owner_identity();
    let mut custody = WorldNativeReadmissionCustody::empty();
    // These are the SAME private catalog/registry/recovery allocations. A late
    // refusal retains any already materialized root and history, not just IDs.
    custody.history = Some(root.state.history.clone());
    custody.branches = Some(root.state.branches.clone());
    custody.recovery = Some(root.state.recovery.clone());
    let result = (|| {
        let signal = root
            .state
            .signal
            .basis_port()
            .readmit_recorded_initial_empty_root(&components.signal)
            .map_err(Denial::SignalInitialRoot)?;
        let bridge = root
            .state
            .bridge
            .readmit_recorded_runtime_world_baseline(&components.bridge, &signal)
            .map_err(Denial::BridgeBaseline)?;
        custody.root = Some(
            authority
                .admit_recorded_component(execution, native, &components.relational)
                .map_err(|cause| Denial::NativeComponent(Box::new(cause)))?,
        );
        let mut descriptors = BTreeMap::from([(head.commit, components.relational.clone())]);
        let mut published = BTreeSet::new();
        let mut occurrences = Vec::new();
        for artifact in &recovered.occurrences {
            let WorldRecord::Occurrence {
                open, relational, ..
            } = &artifact.record
            else {
                unreachable!("occurrence frame")
            };
            let effect = effects
                .remove(&open.association.attempt())
                .ok_or(Denial::OccurrenceEvidenceMismatch)?;
            let subject = effect.subject();
            if subject.receipt().commit_id.0 != relational.commit
                || subject.position().0 != relational.position
                || subject.artifact_digest() != &relational.digest
            {
                return Err(Denial::OccurrenceEvidenceMismatch);
            }
            let before = descriptors
                .get(&open.expected.commit)
                .ok_or(Denial::MissingPredecessor)?;
            let after = authority
                .recorded_publication_successor(execution, native, &effect)
                .map_err(Denial::NativeSuccessor)?;
            let admitted = authority
                .admit_recorded_publication(execution, native, effect, before, &after)
                .map_err(|cause| Denial::NativePublication(Box::new(cause)))?;
            let index = custody.publications.len();
            custody.publications.push(Some(admitted));
            descriptors.insert(open.commit, after);
            published.insert(open.association.attempt());
            occurrences.push((artifact, index));
        }
        let mut opens = Vec::new();
        for open in recovered.opens.values() {
            if published.contains(&open.association.attempt())
                || matches!(
                    recovered.dispositions.get(&open.association.attempt()),
                    Some(RecoveredCustodyDisposition::Released(_))
                )
            {
                continue;
            }
            let before = descriptors
                .get(&open.expected.commit)
                .ok_or(Denial::MissingPredecessor)?;
            let admitted = if let Some(effect) = effects.remove(&open.association.attempt()) {
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
            opens.push((open, ids, index, slot_index));
        }
        let before = custody.root.take().expect("same original root admission");
        let expected = root.install_readmitted_root(artifact, before, signal, bridge, execution)?;
        custody.observations.push(expected.clone());
        let mut observations = BTreeMap::from([(head.commit, expected)]);
        for (artifact, index) in occurrences {
            let WorldRecord::Occurrence {
                open, relational, ..
            } = &artifact.record
            else {
                unreachable!("occurrence frame")
            };
            let before = observations
                .get(&open.expected.commit)
                .ok_or(Denial::MissingPredecessor)?;
            let identity = native
                .history()
                .immutable_commit_identity(worth_relational::facade::history::CommitId(
                    relational.commit,
                ))
                .ok_or(Denial::OccurrenceEvidenceMismatch)?;
            let admitted = custody.publications[index]
                .take()
                .expect("same acquired occurrence cap");
            let record = match root.prepare_readmitted_occurrence(
                before,
                artifact,
                recovered.occurrence_sources[&open.association.attempt()].clone(),
                admitted,
                identity,
            ) {
                Ok(record) => record,
                Err((admitted, cause)) => {
                    custody.publications[index] = Some(admitted);
                    return Err(cause);
                }
            };
            custody.records.push(record.clone());
            let selected = root.install_readmitted_occurrence(&record, &mut custody.capacities)?;
            custody.observations.push(selected.clone());
            observations.insert(open.commit, selected);
        }
        for (open, (attempt, commit, identity), index, slot_index) in opens {
            let expected = observations
                .get(&open.expected.commit)
                .ok_or(Denial::MissingPredecessor)?
                .clone();
            custody.slots[slot_index]
                .take()
                .expect("same reserved slot")
                .install_recovered(RecoveredWorldPublicationInputs {
                    identity,
                    attempt,
                    commit,
                    open: open.clone(),
                    expected,
                    binding: binding.clone(),
                    native: custody.publications[index].take(),
                });
        }
        Ok(())
    })();
    result.map_err(|cause| custody.refuse(cause))
}
