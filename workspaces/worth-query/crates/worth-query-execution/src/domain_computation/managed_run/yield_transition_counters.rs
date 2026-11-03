#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryYieldTransitionCounters {
    eligibility_classification_count: usize,
    bridge_finalization_attempt_count: usize,
    checkpoint_suspension_attempt_count: usize,
    checkpoint_retained_byte_probe_count: usize,
    retained_resource_validation_count: usize,
    artifact_registry_snapshot_count: usize,
    yielded_capability_mint_count: usize,
}

impl WorthQueryYieldTransitionCounters {
    pub const fn eligibility_classification_count(self) -> usize {
        self.eligibility_classification_count
    }

    pub const fn bridge_finalization_attempt_count(self) -> usize {
        self.bridge_finalization_attempt_count
    }

    pub const fn checkpoint_suspension_attempt_count(self) -> usize {
        self.checkpoint_suspension_attempt_count
    }

    pub const fn checkpoint_retained_byte_probe_count(self) -> usize {
        self.checkpoint_retained_byte_probe_count
    }

    pub const fn retained_resource_validation_count(self) -> usize {
        self.retained_resource_validation_count
    }

    pub const fn artifact_registry_snapshot_count(self) -> usize {
        self.artifact_registry_snapshot_count
    }

    pub const fn yielded_capability_mint_count(self) -> usize {
        self.yielded_capability_mint_count
    }

    pub(super) fn classified_eligibility(&mut self) {
        self.eligibility_classification_count =
            self.eligibility_classification_count.saturating_add(1);
    }

    pub(super) fn attempted_bridge_finalization(&mut self) {
        self.bridge_finalization_attempt_count =
            self.bridge_finalization_attempt_count.saturating_add(1);
    }

    pub(super) fn attempted_checkpoint_suspension(&mut self) {
        self.checkpoint_suspension_attempt_count =
            self.checkpoint_suspension_attempt_count.saturating_add(1);
    }

    pub(super) fn observed_checkpoint_retained_bytes(&mut self, count: usize) {
        self.checkpoint_retained_byte_probe_count = self
            .checkpoint_retained_byte_probe_count
            .saturating_add(count);
    }

    pub(super) fn validated_retained_resources(&mut self) {
        self.retained_resource_validation_count =
            self.retained_resource_validation_count.saturating_add(1);
    }

    pub(super) fn observed_artifact_registry(&mut self) {
        self.artifact_registry_snapshot_count =
            self.artifact_registry_snapshot_count.saturating_add(1);
    }

    pub(super) fn minted_yielded_capability(&mut self) {
        self.yielded_capability_mint_count = self.yielded_capability_mint_count.saturating_add(1);
    }
}
