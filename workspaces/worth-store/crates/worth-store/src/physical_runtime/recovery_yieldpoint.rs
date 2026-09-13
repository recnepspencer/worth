use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

const WAIT_DEADLINE: Duration = Duration::from_secs(60);

/// A named process boundary used by the fresh-process recovery proof.
///
/// The probe is inert unless the recovery entry point explicitly receives it.
/// It is descriptive test instrumentation, not a recovery authority or an
/// input decoded from the Store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryYieldpointStage {
    StagingMaterialization,
    StagingSynchronization,
    CandidateMaterialization,
    CandidateSynchronization,
    RootProtocolReplacement,
    RecordNamespaceSynchronization,
    FreshReopenCurrentSelector,
    FreshReopenRootManifest,
    FreshReopenExactBinding,
    CleanupFreshnessRead,
    CleanupRemoval,
}

impl PhysicalRecoveryYieldpointStage {
    pub fn from_label(label: &str) -> Option<Self> {
        Some(match label {
            "staging-materialization" => Self::StagingMaterialization,
            "staging-synchronization" => Self::StagingSynchronization,
            "candidate-materialization" => Self::CandidateMaterialization,
            "candidate-synchronization" => Self::CandidateSynchronization,
            "root-protocol-replacement" => Self::RootProtocolReplacement,
            "record-namespace-synchronization" => Self::RecordNamespaceSynchronization,
            "fresh-reopen-current-selector" => Self::FreshReopenCurrentSelector,
            "fresh-reopen-root-manifest" => Self::FreshReopenRootManifest,
            "fresh-reopen-exact-binding" => Self::FreshReopenExactBinding,
            "cleanup-freshness-read" => Self::CleanupFreshnessRead,
            "cleanup-removal" => Self::CleanupRemoval,
            _ => return None,
        })
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::StagingMaterialization => "staging-materialization",
            Self::StagingSynchronization => "staging-synchronization",
            Self::CandidateMaterialization => "candidate-materialization",
            Self::CandidateSynchronization => "candidate-synchronization",
            Self::RootProtocolReplacement => "root-protocol-replacement",
            Self::RecordNamespaceSynchronization => "record-namespace-synchronization",
            Self::FreshReopenCurrentSelector => "fresh-reopen-current-selector",
            Self::FreshReopenRootManifest => "fresh-reopen-root-manifest",
            Self::FreshReopenExactBinding => "fresh-reopen-exact-binding",
            Self::CleanupFreshnessRead => "cleanup-freshness-read",
            Self::CleanupRemoval => "cleanup-removal",
        }
    }
}

/// Explicitly armed recovery instrumentation for one child process.
///
/// The parent test waits for `reached`, then kills the child. A release file
/// is also supported so the same seam can be exercised without a kill.
#[derive(Debug, Clone)]
pub struct PhysicalRecoveryProcessYieldpoint {
    stage: PhysicalRecoveryYieldpointStage,
    reached: PathBuf,
    release: PathBuf,
    cancel: PathBuf,
    wait_deadline: Duration,
}

#[must_use = "a cancelled or timed-out recovery yieldpoint must become a recovery outcome"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryYieldpointWaitResult {
    NotArmed,
    Released,
    Cancelled { partial_effect_possible: bool },
    DeadlineExceeded { partial_effect_possible: bool },
}

impl PhysicalRecoveryYieldpointWaitResult {
    pub const fn is_interrupted(self) -> bool {
        matches!(self, Self::Cancelled { .. } | Self::DeadlineExceeded { .. })
    }

    pub const fn partial_effect_possible(self) -> bool {
        match self {
            Self::Cancelled {
                partial_effect_possible,
            }
            | Self::DeadlineExceeded {
                partial_effect_possible,
            } => partial_effect_possible,
            Self::NotArmed | Self::Released => false,
        }
    }
}

impl PhysicalRecoveryProcessYieldpoint {
    pub fn new(
        stage: PhysicalRecoveryYieldpointStage,
        reached: PathBuf,
        release: PathBuf,
        cancel: PathBuf,
    ) -> Self {
        Self::new_with_wait_deadline(stage, reached, release, cancel, WAIT_DEADLINE)
    }

    pub fn new_with_wait_deadline(
        stage: PhysicalRecoveryYieldpointStage,
        reached: PathBuf,
        release: PathBuf,
        cancel: PathBuf,
        wait_deadline: Duration,
    ) -> Self {
        Self {
            stage,
            reached,
            release,
            cancel,
            wait_deadline,
        }
    }

    pub const fn stage(&self) -> PhysicalRecoveryYieldpointStage {
        self.stage
    }

    pub fn pause_after(
        &self,
        stage: PhysicalRecoveryYieldpointStage,
    ) -> PhysicalRecoveryYieldpointWaitResult {
        self.pause_after_with_deadline(stage, self.wait_deadline)
    }

    fn pause_after_with_deadline(
        &self,
        stage: PhysicalRecoveryYieldpointStage,
        wait_deadline: Duration,
    ) -> PhysicalRecoveryYieldpointWaitResult {
        if self.stage != stage {
            return PhysicalRecoveryYieldpointWaitResult::NotArmed;
        }
        std::fs::write(&self.reached, stage.label())
            .unwrap_or_else(|error| panic!("write recovery yieldpoint marker: {error}"));
        let deadline = Instant::now() + wait_deadline;
        while !self.release.exists() {
            if self.cancel.exists() {
                return PhysicalRecoveryYieldpointWaitResult::Cancelled {
                    partial_effect_possible: true,
                };
            }
            if Instant::now() >= deadline {
                return PhysicalRecoveryYieldpointWaitResult::DeadlineExceeded {
                    partial_effect_possible: true,
                };
            }
            thread::sleep(Duration::from_millis(10));
        }
        PhysicalRecoveryYieldpointWaitResult::Released
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_paths(label: &str) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "worth-store-yieldpoint-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock is after the unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("create yieldpoint test directory");
        (
            root.join("reached"),
            root.join("release"),
            root.join("cancel"),
        )
    }

    #[test]
    fn cancellation_is_reported_after_the_named_boundary() {
        let (reached, release, cancel) = test_paths("cancel");
        std::fs::write(&cancel, "cancel").expect("arm cancellation");
        let yieldpoint = PhysicalRecoveryProcessYieldpoint::new(
            PhysicalRecoveryYieldpointStage::StagingMaterialization,
            reached.clone(),
            release,
            cancel,
        );

        assert_eq!(
            yieldpoint.pause_after(PhysicalRecoveryYieldpointStage::StagingMaterialization),
            PhysicalRecoveryYieldpointWaitResult::Cancelled {
                partial_effect_possible: true,
            }
        );
        assert_eq!(
            std::fs::read_to_string(reached).expect("read reached marker"),
            "staging-materialization"
        );
    }

    #[test]
    fn expired_boundary_is_reported_as_partial_effect_possible() {
        let (reached, release, cancel) = test_paths("deadline");
        let yieldpoint = PhysicalRecoveryProcessYieldpoint::new(
            PhysicalRecoveryYieldpointStage::StagingSynchronization,
            reached,
            release,
            cancel,
        );

        assert_eq!(
            yieldpoint.pause_after_with_deadline(
                PhysicalRecoveryYieldpointStage::StagingSynchronization,
                Duration::ZERO,
            ),
            PhysicalRecoveryYieldpointWaitResult::DeadlineExceeded {
                partial_effect_possible: true,
            }
        );
    }

    #[test]
    fn configured_process_deadline_is_distinct_from_cancellation() {
        let (reached, release, cancel) = test_paths("configured-deadline");
        let yieldpoint = PhysicalRecoveryProcessYieldpoint::new_with_wait_deadline(
            PhysicalRecoveryYieldpointStage::CandidateMaterialization,
            reached,
            release,
            cancel,
            Duration::ZERO,
        );

        assert_eq!(
            yieldpoint.pause_after(PhysicalRecoveryYieldpointStage::CandidateMaterialization),
            PhysicalRecoveryYieldpointWaitResult::DeadlineExceeded {
                partial_effect_possible: true,
            }
        );
    }
}
