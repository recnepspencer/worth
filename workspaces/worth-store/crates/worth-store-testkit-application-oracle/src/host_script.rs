//! Total seeded vocabulary; owner-gated operations remain explicit plan entries.
mod generator;
pub use generator::HostScript;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BranchLabel {
    Main,
    Left,
    Right,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointKind {
    Explicit,
    Automatic,
    ImageAdmission,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationActivation {
    Current,
    LifecycleProgramTransition,
    ReopenCheckpointImage,
    LifecycleDurableSibling,
    HomeMaintenanceFunding,
}
impl OperationActivation {
    pub fn owner_merge(self) -> Option<&'static str> {
        match self {
            Self::Current => None,
            Self::LifecycleProgramTransition => {
                Some("P-C: reviewed codex/m3-p3-lifecycle D16 merge")
            }
            Self::ReopenCheckpointImage => Some("P-A: reviewed fab7d6e943 D12 image merge"),
            Self::LifecycleDurableSibling => {
                Some("P-C: reviewed codex/m3-p3-lifecycle sibling-route merge")
            }
            Self::HomeMaintenanceFunding => {
                Some("P-A: reviewed FUNDING LANDED Home maintenance allowance merge")
            }
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostCommand {
    Open,
    Initialize,
    Mutate {
        branch: BranchLabel,
        key: u64,
        value: u64,
    },
    LoseResponse {
        key: u64,
        value: u64,
    },
    Retry {
        key: u64,
        value: u64,
    },
    ConcurrentSameKey {
        key: u64,
        value: u64,
    },
    CreateBranch(BranchLabel),
    MoveProduct {
        branch: BranchLabel,
        key: u64,
        value: u64,
    },
    SettlementFailure {
        key: u64,
        value: u64,
    },
    ContinueCustody {
        key: u64,
        value: u64,
    },
    ReleaseCustody,
    AbandonCaller {
        key: u64,
        value: u64,
    },
    Checkpoint(CheckpointKind),
    Close,
    Reopen,
    ProgramTransition {
        revision: u64,
    },
    ResumeTransition,
}
impl HostCommand {
    pub fn activation(&self) -> OperationActivation {
        match self {
            Self::ProgramTransition { .. } | Self::ResumeTransition => {
                OperationActivation::LifecycleProgramTransition
            }
            Self::Checkpoint(CheckpointKind::ImageAdmission) => {
                OperationActivation::ReopenCheckpointImage
            }
            Self::Checkpoint(CheckpointKind::Automatic) => {
                OperationActivation::HomeMaintenanceFunding
            }
            Self::CreateBranch(_) => OperationActivation::LifecycleDurableSibling,
            Self::Mutate {
                branch: BranchLabel::Left | BranchLabel::Right,
                ..
            }
            | Self::MoveProduct {
                branch: BranchLabel::Left | BranchLabel::Right,
                ..
            } => OperationActivation::LifecycleDurableSibling,
            Self::Open
            | Self::Initialize
            | Self::Mutate {
                branch: BranchLabel::Main,
                ..
            }
            | Self::LoseResponse { .. }
            | Self::Retry { .. }
            | Self::ConcurrentSameKey { .. }
            | Self::MoveProduct {
                branch: BranchLabel::Main,
                ..
            }
            | Self::SettlementFailure { .. }
            | Self::ContinueCustody { .. }
            | Self::ReleaseCustody
            | Self::AbandonCaller { .. }
            | Self::Checkpoint(CheckpointKind::Explicit)
            | Self::Close
            | Self::Reopen => OperationActivation::Current,
        }
    }
}
