use std::collections::{BTreeMap, BTreeSet};

use worth_relational::facade::{
    history::{BranchId, CommitId, RelationalCommitReceipt},
    transactions::RecordRef,
};

const MAX_RETAINED_CONDITIONAL_COMMITS: usize = 100_000;

#[derive(Default)]
struct WorthQueryConditionalCommitRoute {
    branches: BTreeMap<BranchId, WorthQueryConditionalCommitBranchRoute>,
}

#[derive(Default)]
struct WorthQueryConditionalCommitBranchRoute {
    entries: BTreeMap<u64, CommitId>,
    dropped: Option<WorthQueryDroppedConditionalCommitInterval>,
}

#[derive(Clone, Copy)]
struct WorthQueryDroppedConditionalCommitInterval {
    through_sequence: u64,
    minimum_commit: CommitId,
}

impl WorthQueryConditionalCommitRoute {
    fn record(&mut self, sequence: u64, commit: &RelationalCommitReceipt) {
        self.branches
            .entry(commit.branch_id.clone())
            .or_default()
            .record(sequence, commit.commit_id);
    }

    fn branch(&self, branch: &BranchId) -> Option<&WorthQueryConditionalCommitBranchRoute> {
        self.branches.get(branch)
    }
}

impl WorthQueryConditionalCommitBranchRoute {
    fn record(&mut self, sequence: u64, commit: CommitId) {
        self.entries.insert(sequence, commit);
        if self.entries.len() > MAX_RETAINED_CONDITIONAL_COMMITS {
            let dropped = *self
                .entries
                .first_key_value()
                .expect("over-capacity route has a first entry")
                .0;
            let commit = self
                .entries
                .remove(&dropped)
                .expect("the oldest retained route entry exists");
            self.dropped = Some(match self.dropped {
                Some(interval) => WorthQueryDroppedConditionalCommitInterval {
                    through_sequence: dropped,
                    minimum_commit: interval.minimum_commit.min(commit),
                },
                None => WorthQueryDroppedConditionalCommitInterval {
                    through_sequence: dropped,
                    minimum_commit: commit,
                },
            });
        }
    }

    fn requires_reconstruction(&self, cursor: u64, ceiling: Option<CommitId>) -> bool {
        self.dropped.is_some_and(|interval| {
            cursor < interval.through_sequence
                && ceiling.is_some_and(|ceiling| interval.minimum_commit <= ceiling)
        })
    }

    fn after(&self, cursor: u64, maximum: usize) -> impl Iterator<Item = (u64, CommitId)> + '_ {
        self.entries
            .range((
                std::ops::Bound::Excluded(cursor),
                std::ops::Bound::Unbounded,
            ))
            .take(maximum)
            .map(|(sequence, commit)| (*sequence, *commit))
    }
}

#[derive(Default)]
pub(super) struct WorthQueryConditionalCommitJournal {
    latest_sequence: u64,
    exact_routes: BTreeMap<RecordRef, WorthQueryConditionalCommitRoute>,
    whole_graph_route: Option<WorthQueryConditionalCommitRoute>,
    bootstrap_routes: BTreeMap<String, WorthQueryConditionalCommitRoute>,
    completed_bootstrap_routes: BTreeSet<String>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryConditionalCommitBatch {
    pub(in crate::domain_computation::primary_graph) commits: Vec<(u64, CommitId)>,
    pub(in crate::domain_computation::primary_graph) cursor: u64,
    pub(in crate::domain_computation::primary_graph) work_remaining: bool,
    pub(in crate::domain_computation::primary_graph) caught_up_to_latest: bool,
}

impl WorthQueryConditionalCommitJournal {
    pub(super) fn latest_sequence(&self) -> u64 {
        self.latest_sequence
    }

    pub(super) fn replace_routes(
        &mut self,
        exact_records: impl IntoIterator<Item = RecordRef>,
        include_whole_graph: bool,
    ) {
        let retained = exact_records.into_iter().collect::<BTreeSet<_>>();
        self.exact_routes
            .retain(|record, _| retained.contains(record));
        for record in retained {
            self.exact_routes.entry(record).or_default();
        }
        match (include_whole_graph, self.whole_graph_route.is_some()) {
            (true, false) => self.whole_graph_route = Some(Default::default()),
            (false, true) => self.whole_graph_route = None,
            _ => {}
        }
    }

    pub(super) fn replace_bootstrap_routes(
        &mut self,
        bootstrap_identities: impl IntoIterator<Item = String>,
    ) {
        let retained_bootstraps = bootstrap_identities.into_iter().collect::<BTreeSet<_>>();
        self.bootstrap_routes
            .retain(|identity, _| retained_bootstraps.contains(identity));
        for identity in &retained_bootstraps {
            if !self.completed_bootstrap_routes.contains(identity) {
                self.bootstrap_routes.entry(identity.clone()).or_default();
            }
        }
        self.completed_bootstrap_routes
            .retain(|identity| retained_bootstraps.contains(identity));
    }

    pub(super) fn record(
        &mut self,
        commit: &RelationalCommitReceipt,
        records: impl IntoIterator<Item = RecordRef>,
    ) {
        self.latest_sequence = self.latest_sequence.saturating_add(1);
        let sequence = self.latest_sequence;
        let records = records.into_iter().collect::<BTreeSet<_>>();
        if let Some(route) = self.whole_graph_route.as_mut() {
            route.record(sequence, commit);
        }
        for route in self.bootstrap_routes.values_mut() {
            route.record(sequence, commit);
        }
        for record in records {
            if let Some(route) = self.exact_routes.get_mut(&record) {
                route.record(sequence, commit);
            }
        }
    }

    pub(super) fn after_records_with_bootstrap(
        &self,
        branch: &BranchId,
        commit_ceiling: Option<CommitId>,
        sequence: u64,
        maximum: usize,
        watched_records: impl IntoIterator<Item = RecordRef>,
        include_whole_graph: bool,
        bootstrap_identity: Option<&str>,
    ) -> Result<WorthQueryConditionalCommitBatch, &'static str> {
        let watched = watched_records.into_iter().collect::<BTreeSet<_>>();
        let routes = watched
            .iter()
            .filter_map(|record| self.exact_routes.get(record))
            .chain(
                include_whole_graph
                    .then_some(self.whole_graph_route.as_ref())
                    .flatten(),
            )
            .chain(bootstrap_identity.and_then(|identity| self.bootstrap_routes.get(identity)))
            .filter_map(|route| route.branch(branch));
        let mut relevant = BTreeMap::new();
        let mut first_commit_after_ceiling = None;
        for route in routes {
            if route.requires_reconstruction(sequence, commit_ceiling) {
                return Err("conditional authoritative-change route was overrun");
            }
            for (entry_sequence, commit) in route.after(sequence, maximum.saturating_add(1)) {
                if commit_ceiling.is_some_and(|ceiling| commit <= ceiling) {
                    relevant.insert(entry_sequence, commit);
                } else {
                    first_commit_after_ceiling = Some(
                        first_commit_after_ceiling
                            .map_or(entry_sequence, |current: u64| current.min(entry_sequence)),
                    );
                    break;
                }
            }
        }
        let work_remaining = relevant.len() > maximum;
        let commits = relevant.into_iter().take(maximum).collect::<Vec<_>>();
        let cursor = commits
            .last()
            .map(|(sequence, _)| *sequence)
            .unwrap_or_else(|| {
                first_commit_after_ceiling
                    .and_then(|future| future.checked_sub(1))
                    .map_or(self.latest_sequence, |frontier| frontier.max(sequence))
            });
        Ok(WorthQueryConditionalCommitBatch {
            commits,
            cursor,
            work_remaining,
            caught_up_to_latest: cursor == self.latest_sequence,
        })
    }

    #[cfg(test)]
    fn after_records(
        &self,
        branch: &BranchId,
        commit_ceiling: Option<CommitId>,
        sequence: u64,
        maximum: usize,
        watched_records: impl IntoIterator<Item = RecordRef>,
        include_whole_graph: bool,
    ) -> Result<WorthQueryConditionalCommitBatch, &'static str> {
        self.after_records_with_bootstrap(
            branch,
            commit_ceiling,
            sequence,
            maximum,
            watched_records,
            include_whole_graph,
            None,
        )
    }

    pub(super) fn narrow_bootstrap_route_if_current(
        &mut self,
        identity: &str,
        expected_frontier: u64,
    ) -> bool {
        if self.latest_sequence != expected_frontier
            || self.bootstrap_routes.remove(identity).is_none()
        {
            return false;
        }
        self.completed_bootstrap_routes.insert(identity.to_string());
        true
    }
}

#[cfg(test)]
mod tests;
