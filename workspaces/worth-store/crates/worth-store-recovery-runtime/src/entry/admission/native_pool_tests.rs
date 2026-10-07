//! Recovery entry preserves actual native pool admission denial before C8 work.

use std::num::{NonZeroU32, NonZeroU64};

use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalOperationAllocationScope as Scope,
    PhysicalRecordResidencyPolicy, PhysicalRecoveryCoordinationAdmissionError,
    PhysicalResidencyDenial, PhysicalSpeculativeWorkKind as Speculation,
};

use crate::entry::{
    PhysicalRecoveryOpenRequest, PhysicalRecoveryPlatformAuthority, PhysicalRecoveryRefusalKind,
    PhysicalRecoveryStaticConfiguration,
};

#[test]
fn recovery_entry_preserves_native_frame_table_metadata_denial_without_effects() {
    let parent = tempfile::tempdir().expect("native pool admission parent");
    let root = parent.path().join("store");
    super::tests::initialize_store(&root);
    let limits = super::tests::limits(8);
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let format = AdmittedPhysicalRecordFormat::admit(configuration.record_format());
    let bytes = |value| NonZeroU64::new(value).unwrap();
    let count = |value| NonZeroU32::new(value).unwrap();
    // Declaration admission checks dimensions and page geometry. Opening the
    // actual pool must additionally admit its concrete 4096-entry frame table.
    let mut declaration = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(384 << 20))
        .resident_bytes(bytes(64 << 20))
        .metadata_bytes(bytes(1))
        .frame_entries(count(4096))
        .pinned_frames(count(256))
        .pin_leases(count(512))
        .dirty_frames(count(64))
        .dirty_replacement_bytes(bytes(64 << 20))
        .operation_bytes(bytes(256 << 20))
        .progress_headroom_bytes(64 << 10);
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        declaration = declaration.scope_bytes(scope, bytes(256 << 20));
    }
    let policy = declaration
        .speculative_frames(Speculation::Prefetch, count(256))
        .speculative_frames(Speculation::ReadAhead, count(256))
        .speculative_frames(Speculation::WriteBehind, count(64))
        .admit(format)
        .into_result()
        .expect("dimensionally valid finite policy");
    let configuration = configuration
        .with_residency_policy(policy)
        .expect("actual admitted policy has the configured format");
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.clone(), configuration.clone(), limits)
            .expect("genuine persisted Store authority");
    let backend = authority.qualified_backend_profile().clone();
    let request = PhysicalRecoveryOpenRequest::declare(
        root.clone(),
        configuration,
        backend,
        limits,
        authority,
    );
    let refusal = match request.admit() {
        Err(refusal) => refusal,
        Ok(admitted) => {
            let _ = admitted.cancel_before_discovery();
            panic!("native frame metadata must deny recovery entry")
        }
    };
    assert_eq!(
        refusal.kind,
        PhysicalRecoveryRefusalKind::CoordinationAdmission(
            PhysicalRecoveryCoordinationAdmissionError::Residency(
                PhysicalResidencyDenial::MetadataBudgetExceeded,
            ),
        )
    );
    assert_eq!(refusal.recovery_effects(), 0);

    // The failed native constructor must release the real root/session owners.
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.clone(), configuration.clone(), limits)
            .expect("native pool denial releases recovery root ownership");
    let backend = authority.qualified_backend_profile().clone();
    let admitted =
        PhysicalRecoveryOpenRequest::declare(root, configuration, backend, limits, authority)
            .admit()
            .expect("the same Store admits the sufficient metadata counterpart");
    assert_eq!(admitted.counters().recovery_effects, Some(0));
    let _ = admitted.cancel_before_discovery();
}
