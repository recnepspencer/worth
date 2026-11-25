//! Discover only the bounded adjacency footprint before observable read work.
use super::VisibilityProjectionView;
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
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
        crate::execution::request_custody::check_request(execution)?;
        let Some(root) = self.basis.root() else {
            return Ok(());
        };
        let root_source = root.symbol_admitted_projection(&self.symbols);
        if self.is_exact_basis() {
            let source: &dyn crate::storage::overlay::CurrentPartitionAccess =
                self.resident.as_ref().map_or(
                    &root_source as &dyn crate::storage::overlay::CurrentPartitionAccess,
                    |basis| basis,
                );
            probe_head(execution, source, |round| {
                self.discover_adjacency(
                    execution,
                    entities.clone(),
                    demand,
                    || round.has_read_misses(),
                    |context, anchor| {
                        let Some(head) = round.get_current_partition(anchor.partition_id) else {
                            return Ok(AdjacencyIds::default());
                        };
                        let table = head.adjacency(demand.direction);
                        context
                            .checkpoint(table.navigation_work_bound() as u64)
                            .map_err(crate::execution::denial::kernel_stop)?;
                        let Some(set) = table.get(anchor.slot_index()) else {
                            return Ok(AdjacencyIds::default());
                        };
                        context
                            .checkpoint(set.navigation_work_bound() as u64)
                            .map_err(crate::execution::denial::kernel_stop)?;
                        Ok(set.kind_ids(demand.kind))
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
            let basis = AdjacencyKindBasis::of_current_version(
                self.basis.root_version() == Some(self.version_id()),
            );
            probe_full(execution, &root_source, |round| {
                self.discover_adjacency(
                    execution,
                    entities.clone(),
                    demand,
                    || round.has_read_misses(),
                    |context, anchor| {
                        let Some(partition) = round.get_partition(anchor.partition_id) else {
                            return Ok(AdjacencyIds::default());
                        };
                        let table = demand.direction.table(partition);
                        context
                            .checkpoint(table.navigation_work_bound() as u64)
                            .map_err(crate::execution::denial::kernel_stop)?;
                        let Some(set) = table.get(anchor.slot_index()) else {
                            return Ok(AdjacencyIds::default());
                        };
                        context
                            .checkpoint(set.selected_kind_navigation_work_bound(basis) as u64)
                            .map_err(crate::execution::denial::kernel_stop)?;
                        Ok(set.kind_ids(basis, demand.kind))
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
        mut ids: impl FnMut(
            &mut worth_execution::MapKernelContext<'_, '_>,
            EntityId,
        ) -> Result<AdjacencyIds<'source>, Cause>,
        mut accepted: impl FnMut(RelationId, EntityId) -> bool,
    ) -> Result<(), Cause> {
        super::super::read_work::metered_with_memory(
            execution,
            |context, memory| {
                let mut consumed = 0_usize;
                for anchor in entities.clone() {
                    crate::execution::request_custody::check_request(execution)?;
                    if demand.frontier {
                        if demand.exhausted(consumed) {
                            return Ok(());
                        }
                        context
                            .checkpoint(super::super::read_prices::EXAMINE_SLOT)
                            .map_err(crate::execution::denial::kernel_stop)?;
                        consumed = consumed.saturating_add(1);
                    }
                    let mut iterator_bytes = 0_u64;
                    let mut iterator = ids(context, anchor)?.try_iter(|navigation, bytes| {
                        if navigation > 0 {
                            context
                                .checkpoint(navigation)
                                .map_err(crate::execution::denial::kernel_stop)?;
                        }
                        if bytes > 0 {
                            super::super::read_work::record_memory::claim(memory, bytes)?;
                            iterator_bytes = iterator_bytes
                                .checked_add(bytes)
                                .ok_or(Cause::ChargedBytesOverflow)?;
                        }
                        Ok::<_, Cause>(())
                    })?;
                    while let Some(&id) = iterator.try_next(|navigation, bytes| {
                        if navigation > 0 {
                            context
                                .checkpoint(navigation)
                                .map_err(crate::execution::denial::kernel_stop)?;
                        }
                        if bytes > 0 {
                            super::super::read_work::record_memory::claim(memory, bytes)?;
                            iterator_bytes = iterator_bytes
                                .checked_add(bytes)
                                .ok_or(Cause::ChargedBytesOverflow)?;
                        }
                        Ok::<_, Cause>(())
                    })? {
                        crate::execution::request_custody::check_request(execution)?;
                        if demand.exhausted(consumed) {
                            break;
                        }
                        context
                            .checkpoint(super::super::read_prices::EXAMINE_SLOT)
                            .map_err(crate::execution::denial::kernel_stop)?;
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
    pub(super) fn current_matches(
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
    pub(super) fn historical_matches(
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
