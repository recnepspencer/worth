//! Discover only the bounded adjacency footprint before observable read work.
use super::VisibilityProjectionView;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::storage::data::RecordLifecycleState as Lifecycle;
use crate::storage::partition::{AdjacencyDirection, AdjacencyIds, AdjacencyKindBasis};
use crate::storage::residency::{probe_full, probe_head};

#[derive(Clone, Copy)]
pub(super) struct AdjacencyDemand {
    pub(super) kind: KindId,
    pub(super) direction: AdjacencyDirection,
    pub(super) maximum: Option<usize>,
    pub(super) frontier: bool,
    pub(super) include_audit: bool,
}
impl VisibilityProjectionView {
    pub(super) fn demand_adjacency(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entities: impl Clone + IntoIterator<Item = EntityId>,
        demand: AdjacencyDemand,
    ) -> Result<(), Cause> {
        crate::execution::check_request(execution)?;
        if self.resident.is_some() {
            return Ok(());
        }
        let Some(root) = self.selected_root() else {
            return Ok(());
        };
        let source = crate::branch::SymbolAdmittedRootRead::new(root, &self.symbols);
        if self.is_exact_basis() {
            probe_head(execution, &source, |round| {
                self.discover_adjacency(
                    execution,
                    entities.clone(),
                    demand,
                    || round.has_read_misses(),
                    |anchor| {
                        round
                            .get_current_partition(anchor.partition_id)
                            .and_then(|head| {
                                head.adjacency(demand.direction).get(anchor.slot_index())
                            })
                            .map(|set| set.kind_ids(demand.kind))
                            .unwrap_or_default()
                    },
                    |id, anchor| {
                        round
                            .get_current_partition(id.partition_id)
                            .is_some_and(|head| {
                                demand.current_matches(
                                    root.schema_authority().registry(),
                                    head,
                                    id,
                                    anchor,
                                )
                            })
                    },
                )
            })
            .map_err(Cause::from)?
        } else {
            probe_full(execution, &source, |round| {
                self.discover_adjacency(
                    execution,
                    entities.clone(),
                    demand,
                    || round.has_read_misses(),
                    |anchor| {
                        round
                            .get_partition(anchor.partition_id)
                            .and_then(|partition| {
                                demand.direction.table(partition).get(anchor.slot_index())
                            })
                            .map(|set| {
                                set.kind_ids(
                                    AdjacencyKindBasis::of_current_version(
                                        self.basis.root_version() == Some(self.version_id()),
                                    ),
                                    demand.kind,
                                )
                            })
                            .unwrap_or_default()
                    },
                    |id, anchor| {
                        round
                            .get_partition(id.partition_id)
                            .is_some_and(|partition| {
                                demand.historical_matches(
                                    root.schema_authority().registry(),
                                    partition,
                                    id,
                                    anchor,
                                    self.version_id(),
                                )
                            })
                    },
                )
            })
            .map_err(Cause::from)?
        }
    }

    fn discover_adjacency<'source>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entities: impl Clone + IntoIterator<Item = EntityId>,
        demand: AdjacencyDemand,
        has_misses: impl Fn() -> bool,
        mut ids: impl FnMut(EntityId) -> AdjacencyIds<'source>,
        mut accepted: impl FnMut(RelationId, EntityId) -> bool,
    ) -> Result<(), Cause> {
        super::super::read_work::metered_with_memory(
            execution,
            |_, memory| {
                let mut consumed = 0_usize;
                for anchor in entities.clone() {
                    crate::execution::check_request(execution)?;
                    if demand.frontier {
                        if demand.exhausted(consumed) {
                            return Ok(());
                        }
                        consumed = consumed.saturating_add(1);
                    }
                    let mut iterator_bytes = 0;
                    let iterator = ids(anchor).try_iter(|_, bytes| {
                        super::super::read_work::record_memory::claim(memory, bytes)?;
                        iterator_bytes = bytes;
                        Ok::<_, Cause>(())
                    })?;
                    for id in iterator.copied() {
                        crate::execution::check_request(execution)?;
                        if demand.exhausted(consumed) {
                            break;
                        }
                        consumed = consumed.saturating_add(1);
                        let keep = accepted(id, anchor);
                        if has_misses() {
                            break;
                        }
                        if keep {
                            if demand.exhausted(consumed) {
                                break;
                            }
                            consumed = consumed.saturating_add(1);
                        }
                    }
                    super::super::read_work::record_memory::release(memory, iterator_bytes)?;
                    if has_misses() || demand.exhausted(consumed) {
                        return Ok(());
                    }
                }
                Ok(())
            },
            |_| 0,
        )?
    }
}
impl AdjacencyDemand {
    fn current_matches(
        self,
        schema: &crate::schema::data::RelationalSchemaRegistry,
        head: crate::storage::residency::RelationalRegionHeadView<'_>,
        id: RelationId,
        anchor: EntityId,
    ) -> bool {
        let Some(slot) = head.relation_arena().get(&id) else {
            return false;
        };
        slot.kind_id() == Some(self.kind)
            && schema.relation_registration(self.kind).is_ok()
            && self.accepts(slot.lifecycle())
            && slot
                .extra()
                .endpoints
                .as_ref()
                .is_some_and(|ends| self.endpoint(ends, anchor))
    }
    fn historical_matches(
        self,
        schema: &crate::schema::data::RelationalSchemaRegistry,
        partition: &crate::storage::overlay::PartitionState,
        id: RelationId,
        anchor: EntityId,
        version: crate::identity::data::VersionId,
    ) -> bool {
        let Some(metadata) = super::super::visibility::visible_relation_metadata(
            partition,
            id.slot_index(),
            version,
        ) else {
            return false;
        };
        (id.generation.is_zero() || id.generation.0 == metadata.generation)
            && metadata.kind_id == self.kind
            && schema.relation_registration(self.kind).is_ok()
            && self.accepts(super::super::visibility::historical_relation_lifecycle(
                partition,
                id.slot_index(),
                metadata,
                version,
            ))
            && self.endpoint(&metadata.endpoints, anchor)
    }
    fn exhausted(self, consumed: usize) -> bool {
        self.maximum.is_some_and(|maximum| consumed >= maximum)
    }
    fn accepts(self, lifecycle: Lifecycle) -> bool {
        lifecycle == Lifecycle::Live
            || (self.include_audit && lifecycle == Lifecycle::RetainedDanglingForAudit)
    }
    fn endpoint(
        self,
        ends: &crate::storage::substrate::RelationEndpoints,
        anchor: EntityId,
    ) -> bool {
        match self.direction {
            AdjacencyDirection::Outgoing => ends.source == anchor,
            AdjacencyDirection::Incoming => ends.target == anchor,
        }
    }
}
