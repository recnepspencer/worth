use worth_store::physical_runtime::PhysicalDurabilityRecoveryHandoff;
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryStaticConfiguration,
};
use worth_store::physical_runtime::QualifiedPhysicalBackendProfile;

fn declare(
    handoff: PhysicalDurabilityRecoveryHandoff,
    configuration: PhysicalRecoveryStaticConfiguration,
    profile: QualifiedPhysicalBackendProfile,
    limits: PhysicalRecoveryLimits,
    authority: PhysicalRecoveryPlatformAuthority,
) {
    let _ = PhysicalRecoveryOpenRequest::declare(
        handoff,
        configuration,
        profile,
        limits,
        authority,
    );
}

fn main() {}
