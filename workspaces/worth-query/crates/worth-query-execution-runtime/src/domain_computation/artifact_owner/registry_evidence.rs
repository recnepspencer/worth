use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::registry::production_generation;
use super::{WorthQueryArtifactProviderReleasePosture, WorthQueryWorkflowArtifactRegistry};

pub(super) struct WorthQueryArtifactLifecycleSnapshotGate {
    state: RwLock<()>,
}

impl WorthQueryArtifactLifecycleSnapshotGate {
    pub(super) const fn new() -> Self {
        Self {
            state: RwLock::new(()),
        }
    }

    pub(super) fn lifecycle_mutation(&self) -> RwLockReadGuard<'_, ()> {
        self.state
            .read()
            .expect("artifact lifecycle snapshot gate must remain available")
    }

    pub(super) fn evidence_snapshot(&self) -> RwLockWriteGuard<'_, ()> {
        self.state
            .write()
            .expect("artifact lifecycle snapshot gate must remain available")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryWorkflowArtifactRegistryEvidence {
    production_generation: u64,
    produced_artifact_count: usize,
    retained_artifact_count: usize,
    disposed_artifact_count: usize,
    retained_bytes: usize,
    provider_release_complete_count: usize,
    provider_release_pending_count: usize,
    provider_release_recovery_required_count: usize,
}

impl WorthQueryWorkflowArtifactRegistryEvidence {
    pub(super) const fn new(
        production_generation: super::WorthQueryArtifactProductionGeneration,
        produced_artifact_count: usize,
        retained_artifact_count: usize,
        disposed_artifact_count: usize,
        retained_bytes: usize,
        provider_release_complete_count: usize,
        provider_release_pending_count: usize,
        provider_release_recovery_required_count: usize,
    ) -> Self {
        Self {
            production_generation: production_generation.ordinal(),
            produced_artifact_count,
            retained_artifact_count,
            disposed_artifact_count,
            retained_bytes,
            provider_release_complete_count,
            provider_release_pending_count,
            provider_release_recovery_required_count,
        }
    }

    pub const fn production_generation(self) -> u64 {
        self.production_generation
    }

    pub const fn produced_artifact_count(self) -> usize {
        self.produced_artifact_count
    }

    pub const fn retained_artifact_count(self) -> usize {
        self.retained_artifact_count
    }

    pub const fn disposed_artifact_count(self) -> usize {
        self.disposed_artifact_count
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }

    pub const fn provider_release_complete_count(self) -> usize {
        self.provider_release_complete_count
    }

    pub const fn provider_release_pending_count(self) -> usize {
        self.provider_release_pending_count
    }

    pub const fn provider_release_recovery_required_count(self) -> usize {
        self.provider_release_recovery_required_count
    }
}

impl WorthQueryWorkflowArtifactRegistry {
    pub fn evidence(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        let _snapshot = self.snapshot_gate.evidence_snapshot();
        let state = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available");
        let posture = state.posture;
        let owners = state
            .owners
            .values()
            .map(|entry| std::sync::Arc::clone(&entry.lifecycle))
            .collect::<Vec<_>>();
        let mut retained_artifact_count = 0usize;
        let mut disposed_artifact_count = 0usize;
        let mut retained_bytes = 0usize;
        let mut provider_release_complete_count = 0usize;
        let mut provider_release_pending_count = 0usize;
        let mut provider_release_recovery_required_count = 0usize;
        for lifecycle in &owners {
            let snapshot = lifecycle.snapshot();
            if snapshot.is_disposed() {
                disposed_artifact_count = disposed_artifact_count.saturating_add(1);
            } else {
                retained_artifact_count = retained_artifact_count.saturating_add(1);
                retained_bytes = retained_bytes.saturating_add(snapshot.counters().retained_bytes);
            }
            match snapshot.provider_release() {
                WorthQueryArtifactProviderReleasePosture::Retained => {}
                WorthQueryArtifactProviderReleasePosture::Pending => {
                    provider_release_pending_count =
                        provider_release_pending_count.saturating_add(1);
                }
                WorthQueryArtifactProviderReleasePosture::Complete(_) => {
                    provider_release_complete_count =
                        provider_release_complete_count.saturating_add(1);
                }
                WorthQueryArtifactProviderReleasePosture::RecoveryRequired(_) => {
                    provider_release_recovery_required_count =
                        provider_release_recovery_required_count.saturating_add(1);
                }
            }
        }
        WorthQueryWorkflowArtifactRegistryEvidence::new(
            production_generation(posture),
            owners.len(),
            retained_artifact_count,
            disposed_artifact_count,
            retained_bytes,
            provider_release_complete_count,
            provider_release_pending_count,
            provider_release_recovery_required_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::WorthQueryArtifactLifecycleSnapshotGate;

    #[test]
    fn evidence_snapshot_excludes_every_lifecycle_mutation() {
        let gate = WorthQueryArtifactLifecycleSnapshotGate::new();
        let mutation = gate.lifecycle_mutation();
        assert!(gate.state.try_write().is_err());
        drop(mutation);

        let snapshot = gate.evidence_snapshot();
        assert!(gate.state.try_read().is_err());
        drop(snapshot);

        assert!(gate.state.try_read().is_ok());
        assert!(gate.state.try_write().is_ok());
    }
}
