#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationCrashStage {
    BeforePublication,
    DuringPublication,
    AfterPublication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublicationRecoveryFaultInjection {
    None,
    MixedTree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveredPublicationStructureKind {
    OldStableStructure,
    NewStableStructure,
    MixedOldAndNewStructure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveredPublicationStructure {
    kind: RecoveredPublicationStructureKind,
    old_root_epoch: u64,
    old_manifest_epoch: u64,
    new_root_epoch: u64,
    new_manifest_epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicationRecoveryReplayInput {
    stage: PublicationCrashStage,
    fault_injection: PublicationRecoveryFaultInjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutedPublicationRecoveryReceipt {
    stage: PublicationCrashStage,
    recovered_kind: RecoveredPublicationStructureKind,
    recovery_replayed_frames: usize,
}

impl RecoveredPublicationStructure {
    pub const fn old_stable_for_publication_admission(
        root_epoch: u64,
        manifest_epoch: u64,
    ) -> Self {
        Self::old_stable(root_epoch, manifest_epoch)
    }

    pub const fn new_stable_for_publication_admission(
        root_epoch: u64,
        manifest_epoch: u64,
    ) -> Self {
        Self::new_stable(root_epoch, manifest_epoch)
    }

    const fn old_stable(root_epoch: u64, manifest_epoch: u64) -> Self {
        Self {
            kind: RecoveredPublicationStructureKind::OldStableStructure,
            old_root_epoch: root_epoch,
            old_manifest_epoch: manifest_epoch,
            new_root_epoch: root_epoch,
            new_manifest_epoch: manifest_epoch,
        }
    }

    const fn new_stable(root_epoch: u64, manifest_epoch: u64) -> Self {
        Self {
            kind: RecoveredPublicationStructureKind::NewStableStructure,
            old_root_epoch: root_epoch,
            old_manifest_epoch: manifest_epoch,
            new_root_epoch: root_epoch,
            new_manifest_epoch: manifest_epoch,
        }
    }

    pub const fn kind(self) -> RecoveredPublicationStructureKind {
        self.kind
    }

    pub const fn stable_root_epoch(self) -> Option<u64> {
        match self.kind {
            RecoveredPublicationStructureKind::OldStableStructure => Some(self.old_root_epoch),
            RecoveredPublicationStructureKind::NewStableStructure => Some(self.new_root_epoch),
            RecoveredPublicationStructureKind::MixedOldAndNewStructure => None,
        }
    }

    pub const fn stable_manifest_epoch(self) -> Option<u64> {
        match self.kind {
            RecoveredPublicationStructureKind::OldStableStructure => Some(self.old_manifest_epoch),
            RecoveredPublicationStructureKind::NewStableStructure => Some(self.new_manifest_epoch),
            RecoveredPublicationStructureKind::MixedOldAndNewStructure => None,
        }
    }

    pub const fn old_root_epoch(self) -> u64 {
        self.old_root_epoch
    }

    pub const fn old_manifest_epoch(self) -> u64 {
        self.old_manifest_epoch
    }

    pub const fn new_root_epoch(self) -> u64 {
        self.new_root_epoch
    }

    pub const fn new_manifest_epoch(self) -> u64 {
        self.new_manifest_epoch
    }
}

impl PublicationRecoveryReplayInput {
    pub const fn from_crash_stage(stage: PublicationCrashStage) -> Self {
        Self {
            stage,
            fault_injection: PublicationRecoveryFaultInjection::None,
        }
    }

    pub const fn mixed_tree_fault_attempt(stage: PublicationCrashStage) -> Self {
        Self {
            stage,
            fault_injection: PublicationRecoveryFaultInjection::MixedTree,
        }
    }

    pub const fn execute(
        self,
        recovery_replayed_frames: usize,
    ) -> ExecutedPublicationRecoveryReceipt {
        ExecutedPublicationRecoveryReceipt {
            stage: self.stage,
            recovered_kind: self.recover_structure_kind(),
            recovery_replayed_frames,
        }
    }

    const fn recover_structure_kind(self) -> RecoveredPublicationStructureKind {
        match self.fault_injection {
            PublicationRecoveryFaultInjection::MixedTree => {
                RecoveredPublicationStructureKind::MixedOldAndNewStructure
            }
            PublicationRecoveryFaultInjection::None => match self.stage {
                PublicationCrashStage::BeforePublication => {
                    RecoveredPublicationStructureKind::OldStableStructure
                }
                PublicationCrashStage::DuringPublication
                | PublicationCrashStage::AfterPublication => {
                    RecoveredPublicationStructureKind::NewStableStructure
                }
            },
        }
    }
}

impl ExecutedPublicationRecoveryReceipt {
    pub const fn stage(self) -> PublicationCrashStage {
        self.stage
    }

    pub const fn recovered_kind(self) -> RecoveredPublicationStructureKind {
        self.recovered_kind
    }

    pub const fn recovery_replayed_frames(self) -> usize {
        self.recovery_replayed_frames
    }
}
