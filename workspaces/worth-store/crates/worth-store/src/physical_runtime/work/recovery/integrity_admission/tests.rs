use std::num::NonZeroU64;

use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_integrity::{
    validate_physical_work_obligation, PhysicalDamageCause,
    PhysicalWorkObligationIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::{admit_bounded_obligation, project_validated_obligation, scope_from_pending_name};
use crate::physical_runtime::work::recovery::effect_obligation::encode_record;
use crate::physical_runtime::work::recovery::observation::{
    PhysicalWorkRecoveryAdmissionCounters, PhysicalWorkRecoveryIngressRejection,
};
use crate::physical_runtime::work::{
    PhysicalOperationIdentity, PhysicalWorkGeneration, PhysicalWorkIdentity,
    PhysicalWorkOperationFamily, PhysicalWorkRecoveryTarget,
};
use crate::physical_runtime::{LifecycleGeneration, RuntimeIdentity};

#[test]
fn wrong_filename_scope_is_rejected_before_owner_interpretation() {
    let store = store_identity();
    let bytes = record(store, 3);
    let scope = scope_from_pending_name(
        store,
        "effect-0000000000000001-0000000000000002-0000000000000004.pending",
    )
    .unwrap();
    let mut counters = PhysicalWorkRecoveryAdmissionCounters::default();

    let rejection = admit_bounded_obligation(scope, &bytes, &mut counters).unwrap_err();

    assert!(matches!(
        rejection,
        PhysicalWorkRecoveryIngressRejection::Integrity(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
        ) if localization.cause() == PhysicalDamageCause::ArtifactIdentityMismatch
    ));
    assert_eq!(counters.owner_interpretation_entries(), 0);
    assert_eq!(counters.rejected_before_owner_interpretation_count(), 1);
}

#[test]
fn equal_distinct_incarnation_cannot_open_owner_projection() {
    let store = store_identity();
    let bytes = record(store, 3);
    let substituted = bytes;
    let scope = scope_from_pending_name(
        store,
        "effect-0000000000000001-0000000000000002-0000000000000003.pending",
    )
    .unwrap();
    let inspected = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_physical_work_obligation(inspected, scope);
    let PhysicalWorkObligationIntegrityValidation::Intact(validated) = validation else {
        panic!("canonical obligation rejected")
    };
    let substituted = UntrustedPhysicalArtifact::from_bounded_bytes(&substituted);

    let mut counters = PhysicalWorkRecoveryAdmissionCounters::default();
    assert!(matches!(
        project_validated_obligation(substituted, validated, &mut counters),
        Err(PhysicalWorkRecoveryIngressRejection::SourceIncarnationMismatch)
    ));
    assert_eq!(counters.owner_interpretation_entries(), 0);
    assert_eq!(counters.rejected_before_owner_interpretation_count(), 1);
}

#[test]
fn pending_name_must_carry_three_nonzero_fixed_width_identities() {
    let store = store_identity();
    for name in [
        "effect-1-0000000000000002-0000000000000003.pending",
        "effect-0000000000000000-0000000000000002-0000000000000003.pending",
        "effect-000000000000000A-0000000000000002-0000000000000003.pending",
        "effect-0000000000000001-0000000000000002.pending",
        "effect-0000000000000001-0000000000000002-0000000000000003.extra.pending",
    ] {
        assert_eq!(
            scope_from_pending_name(store, name),
            Err(PhysicalWorkRecoveryIngressRejection::InvalidPendingName)
        );
    }
}

fn record(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    operation: u64,
) -> [u8; worth_store_physical_format::physical_work_obligation::PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES]
{
    encode_record(
        identity(store, operation),
        PhysicalWorkOperationFamily::DurabilityBarrier,
        PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization,
        None,
    )
}

fn identity(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    operation: u64,
) -> PhysicalWorkIdentity {
    PhysicalWorkIdentity::from_instance_owner(
        store,
        RuntimeIdentity::from_reopened(NonZeroU64::new(1).unwrap()),
        PhysicalWorkGeneration::from_lifecycle(LifecycleGeneration::from_reopened(
            NonZeroU64::new(2).unwrap(),
        )),
        PhysicalOperationIdentity::from_reopened(NonZeroU64::new(operation).unwrap()),
    )
}

fn store_identity() -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x31; 16]).unwrap(),
    )
    .published_identity()
}
