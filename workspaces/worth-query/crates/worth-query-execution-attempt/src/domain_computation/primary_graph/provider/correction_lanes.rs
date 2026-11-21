use std::sync::Arc;

use super::branch_commit_coordination;
use super::WorthQueryPrimaryGraphProvider;

impl WorthQueryPrimaryGraphProvider {
    pub fn correction_commit_lanes(
        &self,
        bindings: impl IntoIterator<
            Item = super::super::application_attempt::WorthQueryApplicationIdempotencyBinding,
        >,
    ) -> Vec<Arc<branch_commit_coordination::WorthQueryCorrectionCommitLane>> {
        let lanes = bindings
            .into_iter()
            .filter_map(|binding| {
                self.correction_commit_lane(&binding)
                    .map(|lane| (*binding.key_identity(), lane))
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        lanes.into_values().collect()
    }

    pub fn correction_commit_lane(
        &self,
        binding: &super::super::application_attempt::WorthQueryApplicationIdempotencyBinding,
    ) -> Option<Arc<branch_commit_coordination::WorthQueryCorrectionCommitLane>> {
        self.branch_commit_coordination
            .corrections
            .for_binding(binding)
    }

    pub fn correction_key_lane(
        &self,
        key: &[u8; 32],
    ) -> Arc<branch_commit_coordination::WorthQueryCorrectionCommitLane> {
        self.branch_commit_coordination.corrections.for_key(key)
    }
}
