use worth_store_physical_backend::{
    ArtifactTreeFailure, CompletedScheduledRecoveryReopenRead, DeniedScheduledRecoveryReopenRead,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::{
    PerformedRecoveryPhysicalEffect, PhysicalRecoveryCoordination, RecoveryFreshReopenAction,
};

mod admission;
mod execution;

pub struct PhysicalRecoveryFreshReopenCommand {
    plan: [u8; 32],
    expected_root: DurablePhysicalRootManifest,
    expected_selector: worth_store_physical_format::DurableRootSelector,
    format: PhysicalRecordFormatDeclaration,
}

pub struct CompletedPhysicalRecoveryFreshReopen {
    root: DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    performed: PerformedRecoveryPhysicalEffect<RecoveryFreshReopenAction>,
}

pub enum PhysicalRecoveryFreshReopenOutcome {
    Completed(CompletedPhysicalRecoveryFreshReopen),
    Denied(PhysicalRecoveryFreshReopenDenial),
}

pub struct PhysicalRecoveryFreshReopenDenial {
    stage: PhysicalRecoveryFreshReopenStage,
    kind: PhysicalRecoveryFreshReopenDenialKind,
    selector: Option<CompletedScheduledRecoveryReopenRead>,
    root: Option<CompletedScheduledRecoveryReopenRead>,
    physical: Option<DeniedScheduledRecoveryReopenRead>,
    integrity: Option<crate::physical_runtime::RootProtocolAdmissionDenial>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryFreshReopenStage {
    CurrentSelector,
    RootManifest,
    ExactBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryFreshReopenDenialKind {
    Submission,
    PreEffect(crate::physical_runtime::PhysicalWorkPreEffectDenial),
    Scheduler(crate::physical_runtime::PhysicalSchedulerDenial),
    Media(ArtifactTreeFailure),
    SchedulerSettlement(crate::physical_runtime::PhysicalWorkSchedulerPosture),
    SignalSettlement(crate::physical_runtime::PhysicalSignalSettlementOutcome),
    Yieldpoint(crate::physical_runtime::PhysicalRecoveryYieldpointWaitResult),
    InvalidSelector,
    InvalidRoot,
    BindingMismatch,
}

impl PhysicalRecoveryFreshReopenCommand {
    pub fn new(
        plan: [u8; 32],
        expected_root: DurablePhysicalRootManifest,
        expected_selector: worth_store_physical_format::DurableRootSelector,
        format: PhysicalRecordFormatDeclaration,
    ) -> Option<Self> {
        (expected_root.generation() != 0
            && expected_selector.root_generation() == expected_root.generation()
            && expected_selector.format() == format)
            .then_some(Self {
                plan,
                expected_root,
                expected_selector,
                format,
            })
    }
}

impl PhysicalRecoveryCoordination {
    pub fn execute_fresh_reopen(
        &self,
        media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
        command: PhysicalRecoveryFreshReopenCommand,
    ) -> PhysicalRecoveryFreshReopenOutcome {
        execution::execute(self, media, command)
    }
}

impl CompletedPhysicalRecoveryFreshReopen {
    pub(super) const fn new(
        root: DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        performed: PerformedRecoveryPhysicalEffect<RecoveryFreshReopenAction>,
    ) -> Self {
        Self {
            root,
            format,
            performed,
        }
    }

    pub const fn root(&self) -> &DurablePhysicalRootManifest {
        &self.root
    }

    pub const fn format(&self) -> PhysicalRecordFormatDeclaration {
        self.format
    }

    pub const fn performed(&self) -> &PerformedRecoveryPhysicalEffect<RecoveryFreshReopenAction> {
        &self.performed
    }

    pub fn fresh_reopen_occurrence(&self) -> &super::RecoveryFreshReopenOccurrence {
        match self.performed.occurrence() {
            super::RecoveryPhysicalEffectOccurrence::FreshReopen(occurrence) => occurrence,
            _ => unreachable!("fresh-reopen action carries fresh-reopen occurrence"),
        }
    }
}

impl PhysicalRecoveryFreshReopenDenial {
    pub(super) const fn new(
        stage: PhysicalRecoveryFreshReopenStage,
        kind: PhysicalRecoveryFreshReopenDenialKind,
        selector: Option<CompletedScheduledRecoveryReopenRead>,
        root: Option<CompletedScheduledRecoveryReopenRead>,
        physical: Option<DeniedScheduledRecoveryReopenRead>,
    ) -> Self {
        Self {
            stage,
            kind,
            selector,
            root,
            physical,
            integrity: None,
        }
    }

    pub(super) const fn with_integrity(
        mut self,
        integrity: crate::physical_runtime::RootProtocolAdmissionDenial,
    ) -> Self {
        self.integrity = Some(integrity);
        self
    }

    pub const fn stage(&self) -> PhysicalRecoveryFreshReopenStage {
        self.stage
    }
    pub const fn kind(&self) -> PhysicalRecoveryFreshReopenDenialKind {
        self.kind
    }
    pub const fn selector(&self) -> Option<&CompletedScheduledRecoveryReopenRead> {
        self.selector.as_ref()
    }
    pub const fn root(&self) -> Option<&CompletedScheduledRecoveryReopenRead> {
        self.root.as_ref()
    }
    pub const fn physical(&self) -> Option<&DeniedScheduledRecoveryReopenRead> {
        self.physical.as_ref()
    }
    pub const fn integrity(&self) -> Option<crate::physical_runtime::RootProtocolAdmissionDenial> {
        self.integrity
    }
}

pub(super) const fn artifact(
    stage: PhysicalRecoveryFreshReopenStage,
    generation: u64,
) -> RecordArtifactFile {
    match stage {
        PhysicalRecoveryFreshReopenStage::CurrentSelector
        | PhysicalRecoveryFreshReopenStage::ExactBinding => RecordArtifactFile::CurrentRootSelector,
        PhysicalRecoveryFreshReopenStage::RootManifest => {
            RecordArtifactFile::RootManifest { generation }
        }
    }
}
