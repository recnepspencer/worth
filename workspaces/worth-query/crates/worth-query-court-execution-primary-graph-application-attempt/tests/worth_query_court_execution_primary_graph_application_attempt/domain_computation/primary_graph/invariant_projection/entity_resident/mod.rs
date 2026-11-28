//! Residency and cloning of a genuinely projected entity identity.

use std::sync::Arc;
use worth_foundational::facade::ChargedBytes;
use worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::{
    AccountIdentity, world_installation::installed_authorization_world,
};

#[test]
fn projected_entity_identity_charges_and_shares_its_key_when_cloned() {
    let world = installed_authorization_world(true, WorthQueryAdmissionClock::system());
    let identity = world
        .application
        .with_host_advancement(|phase| {
            let projected = world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        reader
                            .resolve_entity(
                                AccountIdentity::at("AccountIdentity"),
                                "account-1".to_owned(),
                            )
                            .expect("the installed account is projected")
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            projected.output().clone()
        })
        .unwrap();
    drop(world);

    let key = identity.entity_for_test();
    let weak = Arc::downgrade(key);
    let clone = identity.clone();
    assert!(Arc::ptr_eq(key, clone.entity_for_test()));
    let charged = (2 * size_of::<usize>() + key.len()).next_multiple_of(align_of::<usize>()) as u64;
    assert_eq!(identity.additional_charged_bytes(), charged);
    assert_eq!(clone.additional_charged_bytes(), charged);
    drop(identity);
    assert!(weak.upgrade().is_some());
    drop(clone);
    assert!(weak.upgrade().is_none());
}
