use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound::{Included, Unbounded};

use crate::domain_installation::{
    WorthQueryBoundCollectionWindow, WorthQueryCollectionDeliveryCounters,
    WorthQueryCollectionRowHandle, WorthQueryNativeAccessKey,
};
use crate::memory_workspace::{WorthQueryEntity, WorthQueryEntityIdentity};
use crate::projection_consumption::ConsumedNativeValue;
use crate::runtime::{
    canonical_ordering_key, row_matches_predicates, WorthQueryCanonicalOrderingKey,
};

#[path = "index/fact_projection.rs"]
mod fact_projection;
use fact_projection::{grouping_identity, native_value, patch_facts, PatchFactRequest};
#[path = "index/fresh_row_projection.rs"]
mod fresh_row_projection;
#[path = "index/maintenance_target.rs"]
mod maintenance_target;
use maintenance_target::maintenance_targets;
pub(crate) use maintenance_target::WorthQueryCollectionChangedNativeTarget;
pub(crate) use maintenance_target::WorthQueryCollectionMaintenanceTarget;
#[path = "index/model.rs"]
mod model;
pub(crate) use model::WorthQueryCollectionMaintenanceInputs;
use model::{MaintenanceRow, WorthQueryCollectionGroupTransition};
pub(crate) use model::{WorthQueryCollectionIndexDelta, WorthQueryCollectionIndexPreview};

pub(crate) struct WorthQueryCollectionMaintenanceIndex {
    collection: String,
    request: crate::declarative_live::DeclarativeLiveQueryRequest,
    window_policy: crate::domain_installation::WorthQueryOperationWindowPolicy,
    continuation_posture: crate::domain_installation::WorthQueryOperationContinuationPosture,
    native_keys: Vec<WorthQueryNativeAccessKey>,
    maintenance_targets: Vec<WorthQueryCollectionMaintenanceTarget>,
    grouping_fields: Vec<worth_query_installation::facade::WorthQueryOperationCollectionField>,
    delivery_supported: bool,
    rows: BTreeMap<WorthQueryCanonicalOrderingKey, MaintenanceRow>,
    identities: BTreeMap<WorthQueryEntityIdentity, WorthQueryCanonicalOrderingKey>,
    source_identities: BTreeMap<WorthQueryEntityIdentity, WorthQueryEntityIdentity>,
}

impl WorthQueryCollectionMaintenanceIndex {
    pub(crate) fn build(
        inputs: WorthQueryCollectionMaintenanceInputs<'_>,
        counters: &mut crate::domain_installation::WorthQueryCollectionCapabilityCounters,
    ) -> Self {
        let WorthQueryCollectionMaintenanceInputs {
            request,
            window_policy,
            continuation_posture,
            delivery_supported,
            entities,
            handles,
            native_keys,
            grouping_fields,
        } = inputs;
        let mut rows = BTreeMap::new();
        let mut identities = BTreeMap::new();
        let mut source_identities = BTreeMap::new();
        let handles_by_source = handles
            .iter()
            .map(|handle| (handle.source_row_identity.as_str(), handle))
            .collect::<BTreeMap<_, _>>();
        let maintenance_targets = maintenance_targets(&request, &grouping_fields);
        for entity in entities {
            let source_identity = entity.identity().terminal_projection_for_reporting();
            let handle = handles_by_source
                .get(source_identity.as_str())
                .expect("validated collection identity facts must retain every execution row");
            let key = canonical_ordering_key(&entity, request.ordering());
            let consumer_identity = handle.entity_identity().clone();
            identities.insert(consumer_identity.clone(), key.clone());
            source_identities.insert(entity.identity().clone(), consumer_identity.clone());
            rows.insert(
                key,
                MaintenanceRow {
                    consumer_identity,
                    grouping_identity: grouping_identity(&entity, &grouping_fields),
                    entity,
                    source_row_identity: handle.source_row_identity.clone(),
                    view_local_identity: handle.view_local_identity().to_string(),
                },
            );
            counters.maintenance_rows_indexed += 1;
        }
        Self {
            collection: request.target().to_string(),
            request,
            window_policy,
            continuation_posture,
            native_keys,
            maintenance_targets,
            grouping_fields,
            delivery_supported,
            rows,
            identities,
            source_identities,
        }
    }

    pub(super) const fn window_policy(
        &self,
    ) -> crate::domain_installation::WorthQueryOperationWindowPolicy {
        self.window_policy
    }

    pub(super) const fn continuation_posture(
        &self,
    ) -> crate::domain_installation::WorthQueryOperationContinuationPosture {
        self.continuation_posture
    }

    pub(super) const fn delivery_supported(&self) -> bool {
        self.delivery_supported
    }

    pub(super) fn selects_native_key(&self, key: &WorthQueryNativeAccessKey) -> bool {
        self.native_keys.contains(key)
    }

    pub(super) fn native_value(
        &self,
        identity: &WorthQueryEntityIdentity,
        key: &WorthQueryNativeAccessKey,
    ) -> Option<ConsumedNativeValue> {
        let ordering = self.identities.get(identity)?;
        let row = self.rows.get(ordering)?;
        Some(native_value(&row.entity, key))
    }

    pub(super) fn preview(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: WorthQueryCollectionPreviewRequest<'_>,
        counters: &mut WorthQueryCollectionDeliveryCounters,
    ) -> Result<WorthQueryCollectionIndexPreview, ()> {
        let WorthQueryCollectionPreviewRequest {
            window,
            affected,
            keys,
            workspace,
        } = request;
        let delta = self.delta(execution, affected, workspace, counters)?;
        Ok(self.preview_delta(window, affected, keys, delta, counters))
    }

    pub(super) fn keys_for_change(
        &self,
        broad_change: bool,
        changes: &[WorthQueryCollectionChangedNativeTarget],
    ) -> Vec<WorthQueryNativeAccessKey> {
        self.native_keys
            .iter()
            .filter(|key| {
                broad_change
                    || self.maintenance_targets.iter().any(|target| {
                        target.matches_native_key(key)
                            && changes.iter().any(|change| target.matches_change(change))
                    })
            })
            .cloned()
            .collect()
    }

    pub(super) fn replacement_targets_for_change(
        &self,
        broad_change: bool,
        changes: &[WorthQueryCollectionChangedNativeTarget],
    ) -> Vec<WorthQueryCollectionMaintenanceTarget> {
        self.maintenance_targets
            .iter()
            .filter(|target| {
                broad_change || changes.iter().any(|change| target.matches_change(change))
            })
            .cloned()
            .collect()
    }

    fn preview_delta(
        &self,
        window: &WorthQueryBoundCollectionWindow,
        affected: &BTreeSet<WorthQueryEntityIdentity>,
        keys: &[WorthQueryNativeAccessKey],
        delta: WorthQueryCollectionIndexDelta,
        counters: &mut WorthQueryCollectionDeliveryCounters,
    ) -> WorthQueryCollectionIndexPreview {
        let selected = self.window_rows(window, &delta, counters);
        let has_more = selected.len() > window.admitted_width();
        let selected = selected
            .into_iter()
            .take(window.admitted_width())
            .collect::<Vec<_>>();
        let consumer_affected = affected
            .iter()
            .map(|identity| {
                self.source_identities
                    .get(identity)
                    .cloned()
                    .unwrap_or_else(|| identity.clone())
            })
            .collect::<BTreeSet<_>>();
        let facts = patch_facts(
            PatchFactRequest {
                selected: &selected,
                prior: window,
                affected: &consumer_affected,
                affected_keys: keys,
                selected_keys: &self.native_keys,
            },
            counters,
        );
        let rows = selected
            .into_iter()
            .enumerate()
            .map(|(ordinal, row)| {
                WorthQueryCollectionRowHandle::new(
                    crate::domain_installation::WorthQueryCollectionRowParts {
                        entity_identity: row.consumer_identity,
                        view_local_identity: row.view_local_identity,
                        source_row_identity: row.source_row_identity,
                        row_ordinal: ordinal,
                        capability_identity: window.capability_identity,
                        capability_generation: window.capability_generation,
                    },
                )
            })
            .collect();
        WorthQueryCollectionIndexPreview {
            rows,
            consumer_affected,
            has_more,
            facts,
            delta,
        }
    }

    pub(super) fn apply(&mut self, delta: WorthQueryCollectionIndexDelta) {
        for key in delta.removals {
            if let Some(row) = self.rows.remove(&key) {
                self.identities.remove(&row.consumer_identity);
                self.source_identities.remove(row.entity.identity());
            }
        }
        for (key, row) in delta.upserts {
            self.identities
                .insert(row.consumer_identity.clone(), key.clone());
            self.source_identities
                .insert(row.entity.identity().clone(), row.consumer_identity.clone());
            self.rows.insert(key, row);
        }
    }

    fn delta(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        affected: &BTreeSet<WorthQueryEntityIdentity>,
        workspace: &crate::runtime::WorthQueryWorkspace,
        counters: &mut WorthQueryCollectionDeliveryCounters,
    ) -> Result<WorthQueryCollectionIndexDelta, ()> {
        if !self.request.traversal().is_empty() {
            return Err(());
        }
        let mut unsupported = false;
        let delta = self.delta_from(affected, counters, |identity| {
            match workspace.collection_entity(execution, &self.collection, identity) {
                crate::runtime::WorthQueryBackendEntityLookup::Found(entity) => Some(entity),
                crate::runtime::WorthQueryBackendEntityLookup::Absent => None,
                crate::runtime::WorthQueryBackendEntityLookup::Unsupported => {
                    unsupported = true;
                    None
                }
            }
        });
        (!unsupported).then_some(delta).ok_or(())
    }

    fn delta_from(
        &self,
        affected: &BTreeSet<WorthQueryEntityIdentity>,
        counters: &mut WorthQueryCollectionDeliveryCounters,
        mut fresh: impl FnMut(&WorthQueryEntityIdentity) -> Option<WorthQueryEntity>,
    ) -> WorthQueryCollectionIndexDelta {
        let mut removals = BTreeSet::new();
        let mut upserts = BTreeMap::new();
        let mut group_transitions = Vec::new();
        for identity in affected {
            counters.entity_point_lookups += 1;
            let consumer_identity = self
                .source_identities
                .get(identity)
                .cloned()
                .unwrap_or_else(|| identity.clone());
            let old = self
                .identities
                .get(&consumer_identity)
                .and_then(|key| self.rows.get(key));
            if let Some(key) = self.identities.get(&consumer_identity) {
                removals.insert(key.clone());
            }
            let entity = fresh(identity)
                .filter(|entity| row_matches_predicates(entity, self.request.predicate_filters()));
            let old_grouping = old.map(|row| row.grouping_identity.clone());
            let next_grouping = entity
                .as_ref()
                .map(|entity| grouping_identity(entity, &self.grouping_fields));
            if !self.grouping_fields.is_empty() && old_grouping != next_grouping {
                group_transitions.push(WorthQueryCollectionGroupTransition {
                    entity: consumer_identity.clone(),
                    from: old_grouping,
                    to: next_grouping.clone(),
                });
            }
            let Some(entity) = entity else { continue };
            let source_row_identity = old
                .map(|row| row.source_row_identity.clone())
                .unwrap_or_else(|| identity.terminal_projection_for_reporting());
            let view_local_identity = old
                .map(|row| row.view_local_identity.clone())
                .unwrap_or_else(|| source_row_identity.clone());
            let next_grouping = next_grouping.unwrap_or_default();
            let key = canonical_ordering_key(&entity, self.request.ordering());
            upserts.insert(
                key,
                MaintenanceRow {
                    consumer_identity,
                    entity,
                    source_row_identity,
                    view_local_identity,
                    grouping_identity: next_grouping,
                },
            );
            counters.ordering_index_updates += 1;
        }
        WorthQueryCollectionIndexDelta {
            removals,
            upserts,
            group_transitions,
        }
    }

    fn window_rows(
        &self,
        window: &WorthQueryBoundCollectionWindow,
        delta: &WorthQueryCollectionIndexDelta,
        counters: &mut WorthQueryCollectionDeliveryCounters,
    ) -> Vec<MaintenanceRow> {
        let start = (!window.cursor().is_beginning())
            .then(|| {
                window.rows().first().and_then(|row| {
                    self.identities
                        .get(row.entity_identity())
                        .map(|key| Included(key.clone()))
                })
            })
            .flatten();
        let lower = start.unwrap_or(Unbounded);
        let mut retained = self
            .rows
            .range((lower.clone(), Unbounded))
            .filter(|(key, _)| !delta.removals.contains(*key))
            .peekable();
        let mut inserted = delta.upserts.range((lower, Unbounded)).peekable();
        let mut selected = Vec::new();
        while selected.len() <= window.admitted_width() {
            let next = match (retained.peek(), inserted.peek()) {
                (Some((left, _)), Some((right, _))) if left <= right => {
                    retained.next().map(|(_, row)| row.clone())
                }
                (Some(_), Some(_)) => inserted.next().map(|(_, row)| row.clone()),
                (Some(_), None) => retained.next().map(|(_, row)| row.clone()),
                (None, Some(_)) => inserted.next().map(|(_, row)| row.clone()),
                (None, None) => None,
            };
            let Some(row) = next else { break };
            counters.fresh_window_rows_visited += 1;
            selected.push(row);
        }
        selected
    }
}

impl WorthQueryCollectionIndexDelta {
    pub(super) fn group_transitions(&self) -> &[WorthQueryCollectionGroupTransition] {
        &self.group_transitions
    }
}

pub(super) struct WorthQueryCollectionPreviewRequest<'a> {
    pub window: &'a WorthQueryBoundCollectionWindow,
    pub affected: &'a BTreeSet<WorthQueryEntityIdentity>,
    pub keys: &'a [WorthQueryNativeAccessKey],
    pub workspace: &'a crate::runtime::WorthQueryWorkspace,
}
