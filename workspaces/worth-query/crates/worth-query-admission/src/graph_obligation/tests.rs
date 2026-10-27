use worth_query_installation::facade::{
    WorthQueryInstalledGraphObligationKind as Kind,
    WorthQueryInstalledGraphObligationOwner as Owner,
};

use super::support_admission::owner_requirement_is_exact;
mod resource_access;
mod resource_fixture;

#[test]
fn admission_accepts_only_real_phase_one_semantic_owners() {
    assert!(owner_requirement_is_exact(
        Kind::GraphRead,
        &[Owner::RelationalGraph]
    ));
    assert!(owner_requirement_is_exact(
        Kind::AuthorizationObservation,
        &[
            Owner::RelationalGraph,
            Owner::RuntimeBridgeCorrespondence,
            Owner::SignalPolicy,
        ],
    ));
    assert!(owner_requirement_is_exact(
        Kind::MutationTouch,
        &[Owner::QueryApplicationProgram],
    ));
    assert!(!owner_requirement_is_exact(
        Kind::InvariantExecution,
        &[Owner::QueryApplicationProgram],
    ));
}

#[test]
fn graph_work_plan_identity_is_fixed_width_and_monotonic() {
    let first = super::WorthQueryGraphWorkPlanIdentity::mint().unwrap();
    let second = super::WorthQueryGraphWorkPlanIdentity::mint().unwrap();
    assert!(second.as_u64() > first.as_u64());
    assert_eq!(std::mem::size_of_val(&first), std::mem::size_of::<u64>());
}
