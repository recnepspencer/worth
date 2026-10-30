use worth_store::physical_runtime::PhysicalStore;
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryStaticConfiguration,
};
use worth_store::physical_runtime::QualifiedPhysicalBackendProfile;

fn declare(
    configuration: PhysicalRecoveryStaticConfiguration,
    profile: QualifiedPhysicalBackendProfile,
    limits: PhysicalRecoveryLimits,
    authority: PhysicalRecoveryPlatformAuthority,
) {
    let _ = PhysicalRecoveryOpenRequest::declare(
        PhysicalStore,
        configuration,
        profile,
        limits,
        authority,
    );
}

fn main() {}
