use std::collections::BTreeSet;

use crate::lineage::data::{HistoricalLineageResolution, HistoricalResolutionBoundednessBasis};

use super::historical_walk::BranchScopedHistoricalResolutionRequest;
use super::resolution::{HistoricalTraversalScope, SelectedRootReachableCommitScope};
use super::LineageAccess;

impl<'runtime> LineageAccess<'runtime> {
    pub(crate) fn resolve_record_history_for_observation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<Option<HistoricalLineageResolution>, crate::execution::RelationalExecutionDenialCause>
    {
        let root = observation.selected_root();
        let Some(selected_commit) = observation.commit_id() else {
            self.runtime
                .performance_access()
                .count_lineage_historical_resolution(0, 0, 0, 0, 0, 0);
            return Ok(None);
        };
        let exact_lineage = self
            .runtime
            .read_truth()
            .authoritative_entity_record_for_id_from_exact_state(
                root.as_ref(),
                root.schema_authority().registry(),
                entity_id,
            )
            .and_then(|record| record.lineage_id);
        let mut reachable_scope = SelectedRootReachableCommitScope {
            selected_commit_id: selected_commit,
            selected_branch_id: observation.identity().branch_id().clone(),
            ancestry: None,
        };
        // An entity restored at its original identity holds one lineage per
        // restore epoch. A pinned observation resolves the newest lineage it
        // can reach, so a later restore (a greater lineage) never changes it.
        let (mut candidates, needs_participation, seed_probes) = match exact_lineage {
            Some(lineage_id) => (vec![lineage_id], false, 0),
            None => {
                let (candidates, probes) =
                    self.runtime.lineage.indexed_lineages_for_entity(entity_id);
                (candidates, true, probes)
            }
        };
        // Cold node pages admit epochs in demand order, so the index may hold
        // an older epoch while a newer one is still cold: the observation's
        // own slot lineage joins the candidates before any is tried.
        if needs_participation
            && (candidates.is_empty() || self.runtime.lineage.has_paged_nodes())
            && let Some(slot_lineage) = self.head_slot_lineage(execution, root, entity_id)?
            && !candidates.contains(&slot_lineage)
        {
            candidates.push(slot_lineage);
            candidates.sort_unstable_by(|left, right| right.cmp(left));
        }
        let mut work = (seed_probes, 0_usize);
        let mut tried = BTreeSet::new();
        let mut chosen = None;
        for lineage_id in candidates {
            tried.insert(lineage_id);
            if self.lineage_in_scope(
                execution,
                lineage_id,
                needs_participation,
                &mut reachable_scope,
                &mut work,
            )? {
                chosen = Some(lineage_id);
                break;
            }
        }
        if chosen.is_none()
            && needs_participation
            && let Some(candidate) = self.head_slot_lineage(execution, root, entity_id)?
            && tried.insert(candidate)
            && self.lineage_in_scope(execution, candidate, true, &mut reachable_scope, &mut work)?
        {
            chosen = Some(candidate);
        }
        let (lineage_seed_index_probe_count, lineage_seed_event_visit_count) = work;
        let Some(lineage_id) = chosen else {
            let (nodes, catalog, edges) = reachable_scope.ancestry_work();
            self.runtime
                .performance_access()
                .count_lineage_historical_resolution(
                    lineage_seed_index_probe_count,
                    lineage_seed_event_visit_count,
                    0,
                    nodes,
                    edges,
                    catalog,
                );
            return Ok(None);
        };
        Ok(Some(self.resolve_historical_lineage_in_scope(
            BranchScopedHistoricalResolutionRequest {
                branch_id: observation.identity().branch_id().clone(),
                lineage_id,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
                lineage_seed_index_probe_count,
                lineage_seed_event_visit_count,
            },
            HistoricalTraversalScope::SelectedRootReachable(reachable_scope),
        )))
    }

    /// Demands `lineage_id`'s paged events in scope and, when required,
    /// reports whether it participates in an event reachable from the
    /// observation, adding the index probes and event visits to `work`.
    fn lineage_in_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        lineage_id: crate::identity::data::LineageId,
        needs_participation: bool,
        reachable_scope: &mut SelectedRootReachableCommitScope,
        work: &mut (usize, usize),
    ) -> Result<bool, crate::execution::RelationalExecutionDenialCause> {
        if self.runtime.lineage.has_paged_events() {
            let ancestry = reachable_scope.ancestry(self);
            super::event_demand::demand_historical_events(
                &self.runtime.lineage,
                execution,
                lineage_id,
                ancestry.authoring_branches(),
                Some(ancestry.commit_ids()),
            )?;
        }
        if !needs_participation {
            return Ok(true);
        }
        let lineage_ids = BTreeSet::from([lineage_id]);
        let (relevant_branches, branch_axis_probes) =
            reachable_scope.indexed_branches_for_lineages(self, &lineage_ids, false);
        let (event_positions, branch_lineage_probes) = self
            .runtime
            .lineage
            .branch_event_positions_for_lineages(&relevant_branches, &lineage_ids, false);
        work.0 += branch_axis_probes + branch_lineage_probes;
        Ok(event_positions.into_iter().any(|position| {
            work.1 += 1;
            self.event_position_is_reachable(position, reachable_scope)
        }))
    }

    /// The lineage the observation's head slot for `entity_id` carries, its
    /// node demanded and checked to name exactly this entity.
    fn head_slot_lineage(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: &crate::branch::RelationalBranchRoot,
        entity_id: crate::identity::data::EntityId,
    ) -> Result<
        Option<crate::identity::data::LineageId>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        let head = root.demand_region_head(execution, entity_id.partition_id)?;
        let Some(candidate) = head.as_ref().and_then(|head| {
            let view = head.head_view();
            let slot = view.entity(entity_id)?;
            (slot.generation() == entity_id.generation.0)
                .then_some(slot.extra().lineage_id)
                .flatten()
        }) else {
            return Ok(None);
        };
        let Some(node) = self.runtime.lineage.demand_node(execution, candidate)? else {
            return Err(
                crate::execution::RelationalExecutionDenialCause::StorageUnavailable(
                    crate::storage::data::RelationalStorageUnavailable {
                        subject: crate::storage::data::RelationalStorageSubject::Lineage {
                            first: candidate,
                            last: candidate,
                        },
                        cause: crate::storage::data::RelationalStorageUnavailableCause::Damaged,
                    },
                ),
            );
        };
        if node.node().entity_id() != entity_id {
            return Err(node.reject_binding().into());
        }
        Ok(Some(node.node().lineage_id()))
    }
}
