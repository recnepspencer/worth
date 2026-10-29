pub(crate) use {worth_store::physical_runtime::PhysicalRecoveryYieldpointStage};

pub(crate) use {crate::phase_eight_process::production::harness::ProcessWorld};





pub(crate) const RECOVERY_SEAMS: [PhysicalRecoveryYieldpointStage; 11] = [
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingMaterialization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingSynchronization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CandidateMaterialization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CandidateSynchronization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::RootProtocolReplacement,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::RecordNamespaceSynchronization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenCurrentSelector,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenRootManifest,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenExactBinding,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupFreshnessRead,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval,
];
pub(crate) const RECOVERY_SEAM_OPERATION_COUNT: usize = 65;

pub(crate) fn is_cleanup_stage(stage: PhysicalRecoveryYieldpointStage) -> bool {
    matches!(
        stage,
        worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupFreshnessRead
            | worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval
    )
}