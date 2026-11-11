//! Original admission funds serving evidence before effects and releases refusal custody.
use super::snapshot_family;
use worth_store::physical_runtime::{
    PhysicalRecoveryCoordinationAdmissionError, PhysicalRecoveryRejoinResidentDenial,
};
use worth_store_recovery_runtime::PhysicalRecoveryRefusalKind;

#[test]
fn original_recovery_budget_refuses_serving_evidence_before_effects_and_releases_home() {
    let (world, _, _) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1);
    let retained = world.retained_root();
    let root = retained.path();
    drop(world);
    let before = snapshot_family(root);
    let request = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_memory(root, 1);
    let refusal = match request.admit() {
        Err(refusal) => refusal,
        Ok(admitted) => {
            let _ = admitted.cancel_before_discovery();
            panic!("one original byte cannot fund actual serving evidence");
        }
    };
    assert_eq!(refusal.recovery_effects(), 0);
    let PhysicalRecoveryRefusalKind::CoordinationAdmission(
        PhysicalRecoveryCoordinationAdmissionError::ServingCustodyBacking(
            PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
        ),
    ) = &refusal.kind
    else {
        panic!("wrong original funding refusal: {refusal:?}");
    };
    assert_eq!(*admitted, 1);
    assert!(*required > *admitted);
    assert_eq!(snapshot_family(root), before);
    let admitted = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root)
        .admit().expect("original refusal releases the real home for a funded request");
    assert_eq!(admitted.counters().recovery_effects, Some(0));
    let _ = admitted.cancel_before_discovery();
    assert_eq!(snapshot_family(root), before);
}
