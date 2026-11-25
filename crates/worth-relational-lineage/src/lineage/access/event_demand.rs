//! Discover the reachable event footprint before accepting a lineage traversal.
use crate::contracts::execution::denial_cause::RelationalExecutionDenialCause;
use crate::contracts::execution::request_custody::check_request;
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::events::LineageEventKind;
use crate::runtime::state::subsystems::lineage::{LineageSubsystem, ResidentLineageEventPage};
use std::collections::{BTreeMap, BTreeSet};

pub fn demand_historical_events(
    lineage: &LineageSubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    seed: LineageId,
    branches: &BTreeSet<BranchId>,
    reachable: Option<&BTreeSet<CommitId>>,
) -> Result<Vec<ResidentLineageEventPage>, RelationalExecutionDenialCause> {
    let mut frontier = BTreeSet::from([seed]);
    let mut visited = BTreeSet::new();
    let mut custody = BTreeMap::new();
    while !frontier.is_empty() {
        check_request(execution)?;
        visited.extend(frontier.iter().copied());
        for page in lineage.demand_event_pages(execution, Some(&frontier), Some(branches))? {
            custody.entry(page.member()).or_insert(page);
        }
        let (positions, _) = lineage.branch_event_positions_for_lineages(branches, &frontier, true);
        let mut targets = BTreeSet::new();
        for position in positions {
            if reachable.is_some_and(|ids| {
                lineage
                    .event_publication_commit(position)
                    .is_none_or(|commit| !ids.contains(&commit))
            }) {
                continue;
            }
            let event = lineage
                .event(position)
                .ok_or(crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                    subject: crate::contracts::storage::data::storage_unavailable::RelationalStorageSubject::Lineage {
                        first: seed,
                        last: seed,
                    },
                    cause:
                        crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause::IntegrityMismatch,
                })
                .map_err(RelationalExecutionDenialCause::StorageUnavailable)?;
            if matches!(
                event.kind(),
                LineageEventKind::Replace
                    | LineageEventKind::Split
                    | LineageEventKind::Merge
                    | LineageEventKind::Restore
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
