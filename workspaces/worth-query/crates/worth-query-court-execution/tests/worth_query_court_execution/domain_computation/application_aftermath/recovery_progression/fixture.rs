use worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_execution::facade::domain_computation::{
    WorthQueryRecoveryEffectAuthority, WorthQueryRecoveryHandle,
};
pub(super) use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::recovery_custody::{
    RecoveryAdmission, committed_handle, committed_status_handle, recorded_inverse::outboxless_handle,
};
pub(super) use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;

pub(super) fn world() -> AuthorizationWorld {
    installed_authorization_world(true, WorthQueryAdmissionClock::system())
}

pub(super) fn effect_authority(
    world: &AuthorizationWorld,
    handle: &WorthQueryRecoveryHandle,
    admission: &RecoveryAdmission,
) -> WorthQueryRecoveryEffectAuthority {
    world
        .application
        .admit_recovery_effect_authority(handle, admission)
        .unwrap()
}
