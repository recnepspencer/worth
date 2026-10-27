//! Discover the reachable event footprint before accepting a lineage traversal.
use super::LineageAccess;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::LineageId;
use crate::lineage::data::LineageEventKind;
use crate::runtime::ResidentLineageEventPage;
use std::collections::{BTreeMap, BTreeSet};

impl LineageAccess<'_> {
    pub(super) fn demand_historical_events(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        seed: LineageId,
        branches: &BTreeSet<BranchId>,
        reachable: Option<&BTreeSet<CommitId>>,
    ) -> Result<Vec<ResidentLineageEventPage>, crate::execution::RelationalExecutionDenialCause>
    {
        let mut frontier = BTreeSet::from([seed]);
        let mut visited = BTreeSet::new();
        let mut custody = BTreeMap::new();
        while !frontier.is_empty() {
            crate::execution::check_request(execution)?;
            visited.extend(frontier.iter().copied());
            for page in self.runtime.lineage.demand_event_pages(
                execution,
                Some(&frontier),
                Some(branches),
            )? {
                custody.entry(page.member()).or_insert(page);
            }
            let (positions, _) = self
                .runtime
                .lineage
                .branch_event_positions_for_lineages(branches, &frontier, true);
            let mut targets = BTreeSet::new();
            for position in positions {
                if reachable.is_some_and(|ids| {
                    self.runtime
                        .lineage
                        .event_publication_commit(position)
                        .is_none_or(|commit| !ids.contains(&commit))
                }) {
                    continue;
                }
                let event = self
                    .runtime
                    .lineage
                    .event(position)
                    .expect("admitted event index has its immutable row");
                if matches!(
                    event.kind(),
                    LineageEventKind::Replace | LineageEventKind::Split | LineageEventKind::Merge
                ) {
                    targets.extend(
                        event
                            .targets()
                            .iter()
                            .filter(|id| !visited.contains(id))
                            .copied(),
                    );
                }
            }
            frontier = targets;
        }
        Ok(custody.into_values().collect())
    }
}
