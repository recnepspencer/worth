#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryReadmissionCounters {
    preflight_check_count: usize,
    fresh_resource_attempt_count: usize,
    bridge_readmission_attempt_count: usize,
    provider_restore_attempt_count: usize,
    artifact_generation_attempt_count: usize,
    artifact_generation_commit_count: usize,
    committed_attempt_count: usize,
}

impl WorthQueryReadmissionCounters {
    pub(super) fn checked_preflight(&mut self) {
        self.preflight_check_count = self.preflight_check_count.saturating_add(1);
    }

    pub(super) fn minted_fresh_resource_attempt(&mut self) {
        self.fresh_resource_attempt_count = self.fresh_resource_attempt_count.saturating_add(1);
    }

    pub(super) fn attempted_bridge_readmission(&mut self) {
        self.bridge_readmission_attempt_count =
            self.bridge_readmission_attempt_count.saturating_add(1);
    }

    pub(super) fn attempted_provider_restore(&mut self) {
        self.provider_restore_attempt_count = self.provider_restore_attempt_count.saturating_add(1);
    }

    pub(super) fn attempted_artifact_generation(&mut self) {
        self.artifact_generation_attempt_count =
            self.artifact_generation_attempt_count.saturating_add(1);
    }

    pub(super) fn committed_artifact_generation(&mut self) {
        self.artifact_generation_commit_count =
            self.artifact_generation_commit_count.saturating_add(1);
    }

    pub(super) fn committed_attempt(&mut self) {
        self.committed_attempt_count = self.committed_attempt_count.saturating_add(1);
    }

    pub const fn preflight_check_count(self) -> usize {
        self.preflight_check_count
    }

    pub const fn fresh_resource_attempt_count(self) -> usize {
        self.fresh_resource_attempt_count
    }

    pub const fn bridge_readmission_attempt_count(self) -> usize {
        self.bridge_readmission_attempt_count
    }

    pub const fn provider_restore_attempt_count(self) -> usize {
        self.provider_restore_attempt_count
    }

    pub const fn artifact_generation_attempt_count(self) -> usize {
        self.artifact_generation_attempt_count
    }

    pub const fn artifact_generation_commit_count(self) -> usize {
        self.artifact_generation_commit_count
    }

    pub const fn committed_attempt_count(self) -> usize {
        self.committed_attempt_count
    }
}
