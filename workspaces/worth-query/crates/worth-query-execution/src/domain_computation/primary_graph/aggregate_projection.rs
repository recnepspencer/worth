use std::collections::{BTreeMap, BTreeSet};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[derive(Debug)]
enum RefreshReadDenial {
    Request(Cause),
    Unavailable,
}
impl From<Cause> for RefreshReadDenial {
    fn from(cause: Cause) -> Self {
        Self::Request(cause)
    }
}

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, KindId, VersionId};
use worth_relational::facade::publication::PublishedAuthoritativeRecordPatch;
use worth_relational::facade::runtime::{
    ProjectionAspectScope, RelationalRuntime, VisibilityProjectionView,
};
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::transactions::RecordRef;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct WorthQueryIncomingSumKey {
    pub(super) relation_kind: KindId,
    pub(super) source_kind: KindId,
    pub(super) target_kind: KindId,
    pub(super) field: AspectFieldLocator,
}

#[derive(Clone, Default)]
pub(super) struct WorthQueryAggregateProjections {
    incoming_sums: BTreeMap<WorthQueryIncomingSumKey, WorthQueryIncomingSumGeneration>,
}

#[derive(Clone)]
struct WorthQueryIncomingSumGeneration {
    version: VersionId,
    materialized: BTreeMap<EntityId, WorthQueryIncomingAggregate>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryIncomingAggregate {
    pub(super) sum: i64,
    pub(super) source_count: u64,
}

impl WorthQueryAggregateProjections {
    pub(super) fn cached_aggregate(
        &mut self,
        key: &WorthQueryIncomingSumKey,
        target: EntityId,
        version: VersionId,
    ) -> Option<WorthQueryIncomingAggregate> {
        let generation = self.incoming_sums.get_mut(key)?;
        if generation.version != version {
            generation.materialized.clear();
            generation.version = version;
            return None;
        }
        generation.materialized.get(&target).copied()
    }

    pub(super) fn retain_aggregate(
        &mut self,
        key: WorthQueryIncomingSumKey,
        target: EntityId,
        version: VersionId,
        sum: i64,
        source_count: u64,
    ) {
        let generation =
            self.incoming_sums
                .entry(key)
                .or_insert_with(|| WorthQueryIncomingSumGeneration {
                    version,
                    materialized: BTreeMap::new(),
                });
        if generation.version != version {
            generation.materialized.clear();
            generation.version = version;
        }
        generation
            .materialized
            .insert(target, WorthQueryIncomingAggregate { sum, source_count });
    }

    pub(super) fn refresh_after_commit(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &mut RelationalRuntime,
        before: &SnapshotHandle,
        after: &SnapshotHandle,
        patches: &[PublishedAuthoritativeRecordPatch],
    ) -> Result<(), Cause> {
        let result = (|| {
            for (key, generation) in &mut self.incoming_sums {
                if generation.version != before.version_id() {
                    generation.materialized.clear();
                    generation.version = after.version_id();
                    continue;
                }
                let result = (|| {
                    let sources =
                        affected_sources(execution, runtime, key, before, after, patches)?;
                    for source in sources {
                        let old = source_contributions(execution, runtime, key, before, source)?;
                        let new = source_contributions(execution, runtime, key, after, source)?;
                        update_materialized_targets(&mut generation.materialized, old, new);
                    }
                    Ok::<_, RefreshReadDenial>(())
                })();
                generation.version = after.version_id();
                match result {
                    Ok(()) => {}
                    Err(RefreshReadDenial::Unavailable) => generation.materialized.clear(),
                    Err(RefreshReadDenial::Request(cause)) => return Err(cause),
                }
            }
            Ok(())
        })();
        if result.is_err() {
            self.recover_after_commit(after.version_id());
        }
        result
    }

    pub(super) fn recover_after_commit(&mut self, version: VersionId) {
        for generation in self.incoming_sums.values_mut() {
            generation.materialized.clear();
            generation.version = version;
        }
    }
}

/// Every source a patch touched on the committing branch, before or after.
/// Refused reads or unavailable branch roots invalidate the cached generation.
fn affected_sources(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &WorthQueryIncomingSumKey,
    before: &SnapshotHandle,
    after: &SnapshotHandle,
    patches: &[PublishedAuthoritativeRecordPatch],
) -> Result<BTreeSet<EntityId>, RefreshReadDenial> {
    let truth = runtime.read_truth();
    let views = [
        truth
            .project_snapshot(before, execution)
            .ok_or(RefreshReadDenial::Unavailable)?,
        truth
            .project_snapshot(after, execution)
            .ok_or(RefreshReadDenial::Unavailable)?,
    ];
    let mut sources = BTreeSet::new();
    for patch in patches {
        match patch.target {
            RecordRef::Entity(entity) => {
                for view in &views {
                    if live_entity_kind(execution, view, entity)? == Some(key.source_kind) {
                        sources.insert(entity);
                        break;
                    }
                }
            }
            RecordRef::Relation(relation) => {
                for view in &views {
                    if let Some(source) = view.relation_record_with_projection_scope(
                        execution,
                        relation,
                        ProjectionAspectScope::empty(),
                        |record| {
                            (record.lifecycle() == RecordLifecycleState::Live
                                && record.kind_id() == key.relation_kind)
                                .then_some(record.source())
                        },
                    )? {
                        sources.insert(source);
                    }
                }
            }
        }
    }
    Ok(sources)
}

fn source_contributions(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &WorthQueryIncomingSumKey,
    snapshot: &SnapshotHandle,
    source: EntityId,
) -> Result<BTreeMap<EntityId, i64>, RefreshReadDenial> {
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot, execution)
        .ok_or(RefreshReadDenial::Unavailable)?;
    if live_entity_kind(execution, &view, source)? != Some(key.source_kind) {
        return Ok(BTreeMap::new());
    }
    let Some(AspectValue::Int64(value)) = super::application_attempt::observe_field_value(
        runtime,
        execution,
        snapshot,
        source,
        key.source_kind,
        &key.field,
    )?
    else {
        return Err(RefreshReadDenial::Unavailable);
    };
    let relations = view.outgoing_relations_of_kind(execution, source, key.relation_kind)?;
    if relations.len() > 1 {
        return Err(RefreshReadDenial::Unavailable);
    }
    let mut contributions = BTreeMap::new();
    for relation in relations {
        if live_entity_kind(execution, &view, relation.target)? == Some(key.target_kind) {
            let entry = contributions.entry(relation.target).or_insert(0_i64);
            *entry = entry
                .checked_add(value)
                .ok_or(RefreshReadDenial::Unavailable)?;
        }
    }
    Ok(contributions)
}

/// The kind of a live entity on the view's own branch root.
fn live_entity_kind(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &VisibilityProjectionView,
    entity: EntityId,
) -> Result<Option<KindId>, RefreshReadDenial> {
    view.entity_record_with_projection_scope(
        execution,
        entity,
        ProjectionAspectScope::empty(),
        |record| (record.lifecycle() == RecordLifecycleState::Live).then_some(record.kind_id()),
    )
    .map_err(RefreshReadDenial::Request)
}

fn update_materialized_targets(
    materialized: &mut BTreeMap<EntityId, WorthQueryIncomingAggregate>,
    old: BTreeMap<EntityId, i64>,
    new: BTreeMap<EntityId, i64>,
) {
    let targets = old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for target in targets {
        let Some(current) = materialized.get(&target).copied() else {
            continue;
        };
        let sum = current
            .sum
            .checked_sub(old.get(&target).copied().unwrap_or_default())
            .and_then(|value| value.checked_add(new.get(&target).copied().unwrap_or_default()));
        let source_count = current
            .source_count
            .checked_sub(u64::from(old.contains_key(&target)))
            .and_then(|count| count.checked_add(u64::from(new.contains_key(&target))));
        match (sum, source_count) {
            (Some(sum), Some(source_count)) => {
                materialized.insert(target, WorthQueryIncomingAggregate { sum, source_count });
            }
            _ => {
                materialized.remove(&target);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{update_materialized_targets, WorthQueryIncomingAggregate};
    use std::collections::BTreeMap;
    use worth_relational::facade::identity::{EntityId, PartitionId};

    #[test]
    fn incremental_overflow_evicts_the_reconstructible_target() {
        let target = EntityId::new(PartitionId::new(1), 1, 1);
        let mut materialized = BTreeMap::from([(
            target,
            WorthQueryIncomingAggregate {
                sum: i64::MAX,
                source_count: 1,
            },
        )]);

        update_materialized_targets(
            &mut materialized,
            BTreeMap::new(),
            BTreeMap::from([(target, 1)]),
        );

        assert!(!materialized.contains_key(&target));
    }

    #[test]
    fn incremental_retargeting_updates_sum_and_source_count_together() {
        let old_target = EntityId::new(PartitionId::new(1), 1, 1);
        let new_target = EntityId::new(PartitionId::new(1), 2, 1);
        let mut materialized = BTreeMap::from([
            (
                old_target,
                WorthQueryIncomingAggregate {
                    sum: 7,
                    source_count: 1,
                },
            ),
            (
                new_target,
                WorthQueryIncomingAggregate {
                    sum: 3,
                    source_count: 1,
                },
            ),
        ]);

        update_materialized_targets(
            &mut materialized,
            BTreeMap::from([(old_target, 7)]),
            BTreeMap::from([(new_target, 7)]),
        );

        assert_eq!(
            materialized.get(&old_target),
            Some(&WorthQueryIncomingAggregate {
                sum: 0,
                source_count: 0,
            })
        );
        assert_eq!(
            materialized.get(&new_target),
            Some(&WorthQueryIncomingAggregate {
                sum: 10,
                source_count: 2,
            })
        );
    }
}
