use serde::{Deserialize, Serialize};

use super::{
    DiagnosticsArtifactKind, DiagnosticsDeliveryClass, DiagnosticsScope, RelationalArtifactPolicy,
    RelationalDiagnosticArtifact,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalDiagnosticsProfile {
    pub capture_failures: bool,
    pub capture_rollbacks: bool,
    pub capture_comparisons: bool,
    pub detailed_traces_enabled: bool,
    pub collect_all_invariant_failures: bool,
    pub max_entries_per_artifact: usize,
    pub allow_deferred_hot_artifacts: bool,
    pub allow_reconstructable_hot_artifacts: bool,
}

impl Default for RelationalDiagnosticsProfile {
    fn default() -> Self {
        Self {
            capture_failures: true,
            capture_rollbacks: true,
            capture_comparisons: true,
            detailed_traces_enabled: false,
            collect_all_invariant_failures: false,
            max_entries_per_artifact: 256,
            allow_deferred_hot_artifacts: true,
            allow_reconstructable_hot_artifacts: true,
        }
    }
}

impl RelationalDiagnosticsProfile {
    pub fn geometry_operational_hot_path() -> Self {
        Self {
            capture_failures: true,
            capture_rollbacks: true,
            capture_comparisons: false,
            detailed_traces_enabled: false,
            collect_all_invariant_failures: false,
            max_entries_per_artifact: 64,
            allow_deferred_hot_artifacts: false,
            allow_reconstructable_hot_artifacts: false,
        }
    }

    pub fn geometry_rich_certification() -> Self {
        Self {
            capture_failures: true,
            capture_rollbacks: true,
            capture_comparisons: true,
            detailed_traces_enabled: true,
            collect_all_invariant_failures: false,
            max_entries_per_artifact: 768,
            allow_deferred_hot_artifacts: false,
            allow_reconstructable_hot_artifacts: false,
        }
    }

    pub fn chip_operational_hot_path() -> Self {
        Self {
            capture_failures: true,
            capture_rollbacks: true,
            capture_comparisons: false,
            detailed_traces_enabled: false,
            collect_all_invariant_failures: false,
            max_entries_per_artifact: 48,
            allow_deferred_hot_artifacts: false,
            allow_reconstructable_hot_artifacts: false,
        }
    }

    pub fn chip_rich_certification() -> Self {
        Self {
            capture_failures: true,
            capture_rollbacks: true,
            capture_comparisons: true,
            detailed_traces_enabled: true,
            collect_all_invariant_failures: false,
            max_entries_per_artifact: 256,
            allow_deferred_hot_artifacts: true,
            allow_reconstructable_hot_artifacts: true,
        }
    }

    fn scope_is_hot(scope: DiagnosticsScope) -> bool {
        matches!(
            scope,
            DiagnosticsScope::Transaction
                | DiagnosticsScope::Snapshot
                | DiagnosticsScope::PatchPublication
                | DiagnosticsScope::QueryPlanning
                | DiagnosticsScope::Invariant
        )
    }

    pub fn artifact_policy(
        &self,
        scope: DiagnosticsScope,
        kind: DiagnosticsArtifactKind,
    ) -> RelationalArtifactPolicy {
        let delivery_class = match kind {
            DiagnosticsArtifactKind::MinimalSummary => match scope {
                DiagnosticsScope::Replay => DiagnosticsDeliveryClass::ReconstructableFromReplay,
                _ => DiagnosticsDeliveryClass::MustBeHot,
            },
            DiagnosticsArtifactKind::DetailedTrace => DiagnosticsDeliveryClass::CanDefer,
            DiagnosticsArtifactKind::Failure => DiagnosticsDeliveryClass::MustBeHot,
            DiagnosticsArtifactKind::Rollback => DiagnosticsDeliveryClass::MustBeHot,
            DiagnosticsArtifactKind::Comparison => {
                DiagnosticsDeliveryClass::ReconstructableFromReplay
            }
        };

        let mut enabled = match kind {
            DiagnosticsArtifactKind::MinimalSummary => true,
            DiagnosticsArtifactKind::DetailedTrace => self.detailed_traces_enabled,
            DiagnosticsArtifactKind::Failure => self.capture_failures,
            DiagnosticsArtifactKind::Rollback => self.capture_rollbacks,
            DiagnosticsArtifactKind::Comparison => self.capture_comparisons,
        };

        let mut max_entries = match delivery_class {
            DiagnosticsDeliveryClass::MustBeHot => self.max_entries_per_artifact.max(1),
            DiagnosticsDeliveryClass::CanDefer
            | DiagnosticsDeliveryClass::ReconstructableFromReplay => self.max_entries_per_artifact,
        };

        if Self::scope_is_hot(scope) {
            if matches!(delivery_class, DiagnosticsDeliveryClass::CanDefer)
                && !self.allow_deferred_hot_artifacts
            {
                enabled = false;
                max_entries = 0;
            }
            if matches!(
                delivery_class,
                DiagnosticsDeliveryClass::ReconstructableFromReplay
            ) && !self.allow_reconstructable_hot_artifacts
            {
                enabled = false;
                max_entries = 0;
            }
        }

        match kind {
            DiagnosticsArtifactKind::MinimalSummary => {}
            DiagnosticsArtifactKind::DetailedTrace => {
                if !self.detailed_traces_enabled {
                    enabled = false;
                    max_entries = 0;
                }
            }
            DiagnosticsArtifactKind::Failure => {
                if !self.capture_failures {
                    enabled = false;
                    max_entries = 0;
                }
            }
            DiagnosticsArtifactKind::Rollback => {
                if !self.capture_rollbacks {
                    enabled = false;
                    max_entries = 0;
                }
            }
            DiagnosticsArtifactKind::Comparison => {
                if !self.capture_comparisons {
                    enabled = false;
                    max_entries = 0;
                } else if !self.detailed_traces_enabled {
                    max_entries = max_entries.min(64);
                }
            }
        }

        if !enabled {
            max_entries = 0;
        }

        RelationalArtifactPolicy {
            delivery_class,
            enabled,
            max_entries,
        }
    }

    pub fn should_capture_artifact(
        &self,
        scope: DiagnosticsScope,
        kind: DiagnosticsArtifactKind,
    ) -> bool {
        self.artifact_policy(scope, kind).enabled
    }

    pub fn delivery_class(
        &self,
        scope: DiagnosticsScope,
        kind: DiagnosticsArtifactKind,
    ) -> DiagnosticsDeliveryClass {
        self.artifact_policy(scope, kind).delivery_class
    }

    pub fn max_entries_for(&self, scope: DiagnosticsScope, kind: DiagnosticsArtifactKind) -> usize {
        self.artifact_policy(scope, kind).max_entries
    }

    pub fn filter_artifact(
        &self,
        artifact: RelationalDiagnosticArtifact,
    ) -> Option<RelationalDiagnosticArtifact> {
        let mut artifact = artifact.canonicalized();
        let policy = self.artifact_policy(artifact.scope, artifact.kind);
        if !policy.enabled {
            return None;
        }
        artifact.entries.truncate(policy.max_entries);
        if artifact.entries.is_empty() {
            None
        } else {
            Some(artifact)
        }
    }
}
