use std::num::NonZeroU64;

use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

use super::{PhysicalPublicationAdmission, PhysicalPublicationAdmissionDenial, PhysicalRetentionProfile};
use crate::physical_runtime::{
    LifecycleGeneration, PhysicalMutationIdentity, PhysicalOperationIdentity, PhysicalWorkGeneration,
    PhysicalWorkIdentity, RuntimeIdentity,
};

fn identity(operation: u64) -> PhysicalMutationIdentity {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x41; 16]).unwrap(),
    )
    .published_identity();
    PhysicalMutationIdentity::from_reserved_operation(PhysicalWorkIdentity::from_instance_owner(
        store,
        RuntimeIdentity::from_reopened(NonZeroU64::new(1).unwrap()),
        PhysicalWorkGeneration::from_lifecycle(LifecycleGeneration::from_reopened(
            NonZeroU64::new(1).unwrap(),
        )),
        PhysicalOperationIdentity::from_reopened(NonZeroU64::new(operation).unwrap()),
    ))
}

#[test]
fn retained_publication_blocks_the_next_root_change() {
    let profile = PhysicalRetentionProfile::new(100, 4, 40, 1).unwrap();
    let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
    let held = admission
        .register_exclusive_pending(identity(1))
        .expect("the first publication registers");
    held.retain_unresolved();
    assert_eq!(admission.pending_len(), 1);
    match admission.register_exclusive_pending(identity(2)) {
        Err(PhysicalPublicationAdmissionDenial::ScopeConflict(blocker)) => {
            assert_eq!(blocker, identity(1));
        }
        Ok(_) => panic!("an unresolved publication must block the next root change"),
        Err(PhysicalPublicationAdmissionDenial::Growth(_)) => {
            panic!("the blocker is the pending publication, not growth")
        }
    }
}

#[test]
fn sealed_candidate_charge_survives_lease_drop() {
    let profile = PhysicalRetentionProfile::new(100, 4, 40, 1).unwrap();
    let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
    let lease = admission.reserve_candidate(9, 60).unwrap();
    admission.seal_candidate_charge(lease.generation());
    drop(lease);
    assert_eq!(admission.remaining_growth_bytes(), 0);
    let Err(denied) = admission.reserve_candidate(10, 1) else {
        panic!("sealed retained growth must keep denying one-over requests");
    };
    assert_eq!(denied.remaining_bytes, 0);
    assert_eq!(denied.requested_bytes, 1);
}
