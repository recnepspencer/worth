use std::num::NonZeroU64;

use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

use super::super::PhysicalWorkOperationFamily;
use super::effect_obligation::encode_record;
use super::integrity_admission::{admit_bounded_obligation, scope_from_pending_name};
use super::observation::PhysicalWorkRecoveryAdmissionCounters;
use super::PhysicalWorkRecoveryTarget;
use crate::physical_runtime::work::{
    PhysicalOperationIdentity, PhysicalWorkGeneration, PhysicalWorkIdentity,
};
use crate::physical_runtime::{LifecycleGeneration, RuntimeIdentity};

const OPERATION_3_HEX: &str = "575045464645435406060000000000000102030405060708090a0b0c0d0e0f10010000000000000002000000000000000300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000b2a18aee7dbc138ec738507b8fcd7f722280d499d9bf0f3962294fccfe50bd0c";
const OPERATION_4_HEX: &str = "575045464645435406050000000000000102030405060708090a0b0c0d0e0f1001000000000000000200000000000000040000000000000009000000000000000a00000000000000abababababababababababababababababababababababababababababababab0601000000000000070000000000000008000000000000004cad4be6809b3b053cf2951815f20c0d1cb2649b87a61b582c1a0a3a89010b90";

#[test]
fn store_owner_mapping_reproduces_both_frozen_records_and_names() {
    let store = store_identity();
    let cases = [
        (
            identity(store, 3),
            PhysicalWorkOperationFamily::DurabilityBarrier,
            PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization,
            None,
            OPERATION_3_HEX,
        ),
        (
            identity(store, 4),
            PhysicalWorkOperationFamily::WalAppend,
            PhysicalWorkRecoveryTarget::WalArtifactInterval {
                segment: 7,
                generation: 8,
                offset: 9,
                byte_count: 10,
            },
            Some([0xab; 32]),
            OPERATION_4_HEX,
        ),
    ];
    for (identity, family, target, digest, expected_hex) in cases {
        let expected = literal(expected_hex);
        let encoded = encode_record(identity, family, target, digest);
        assert_eq!(encoded.as_slice(), expected);
        let name = format!(
            "effect-{:016x}-{:016x}-{:016x}.pending",
            identity.runtime().get(),
            identity.generation().lifecycle().get(),
            identity.operation().get(),
        );
        let scope = scope_from_pending_name(store, &name).expect("pending name establishes scope");
        let mut counters = PhysicalWorkRecoveryAdmissionCounters::default();
        let decoded = admit_bounded_obligation(scope, &encoded, &mut counters)
            .expect("integrity-admitted Store mapping projects");
        assert_eq!(decoded.family(), family);
        assert_eq!(decoded.target(), target);
        assert_eq!(decoded.payload_digest(), digest);
        assert_eq!(counters.attempted(), 0);
        assert_eq!(counters.admitted_count(), 1);
        assert_eq!(counters.owner_interpretation_entries(), 1);
    }
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
    let proposed = ProposedStoreIdentity::from_nonzero_bytes([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ])
    .unwrap();
    StoreNamespaceIdentityRecord::decode(
        &StoreNamespaceIdentityRecord::new(StoreNamespaceVersion::CURRENT, proposed).encode(),
    )
    .unwrap()
    .published_identity()
}

fn literal(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).expect("literal hex byte")
        })
        .collect()
}
