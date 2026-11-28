use std::sync::Arc;

use crate::facade::{
    RelationalBridgeBranchHeadLease, RelationalBridgeObservationLease, TruthCommitIdentity,
    TruthSnapshotIdentity,
};

use super::{NamedTruth, RelationalFixture};

#[derive(Clone, Debug)]
pub struct ScenarioRelationalTruth {
    pub native: RelationalFixture,
    pub first_commit: TruthCommitIdentity,
    observations: Arc<[RelationalBridgeObservationLease]>,
    heads: Arc<[RelationalBridgeBranchHeadLease]>,
}

impl ScenarioRelationalTruth {
    pub fn new(
        native: RelationalFixture,
        first_commit: TruthCommitIdentity,
        observations: Vec<RelationalBridgeObservationLease>,
        head: Option<RelationalBridgeBranchHeadLease>,
    ) -> Self {
        Self::new_with_heads(
            native,
            first_commit,
            observations,
            head.into_iter().collect(),
        )
    }

    pub fn new_with_heads(
        native: RelationalFixture,
        first_commit: TruthCommitIdentity,
        observations: Vec<RelationalBridgeObservationLease>,
        heads: Vec<RelationalBridgeBranchHeadLease>,
    ) -> Self {
        Self {
            native,
            first_commit,
            observations: observations.into(),
            heads: heads.into(),
        }
    }

    pub fn snapshot(&self) -> &TruthSnapshotIdentity {
        self.heads.first().map_or_else(
            || self.observations[0].snapshot_identity(),
            |head| head.snapshot_identity(),
        )
    }

    pub fn named(name: &str, bind_head: bool) -> (Self, TruthCommitIdentity) {
        let truth = NamedTruth::new();
        let (commit, observation) = truth.commit_initial_name(name);
        let head = bind_head.then(|| truth.bind_main_head());
        (
            Self::new(truth.native(), commit.clone(), vec![observation], head),
            commit,
        )
    }
}
