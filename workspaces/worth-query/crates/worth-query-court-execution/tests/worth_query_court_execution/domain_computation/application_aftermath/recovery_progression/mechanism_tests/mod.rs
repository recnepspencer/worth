//! Handle-owned mechanism and authority-axis evidence.

use worth_query_installation::facade::WorthQueryInstalledAftermathContract;

use worth_query_execution::facade::domain_computation::compensate_recovery_handle;
use worth_query_execution::facade::domain_computation::reconcile_recovery_handle;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryEffectAuthority;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture as fixture;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandle;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::application_aftermath::recovery_handle::binding::WorthQueryRecoveryHandleBindingAxisProbe;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandleDenialKind;

pub fn probe_handle(aftermath: WorthQueryInstalledAftermathContract) -> WorthQueryRecoveryHandle {
    let binding = WorthQueryRecoveryHandleBindingAxisProbe::real()
        .installed_aftermath(aftermath)
        .expires_at_unix_ms(Some(u64::MAX))
        .finish();
    WorthQueryRecoveryHandle::axis_probe_for_test(binding)
}

pub fn authority(handle: &WorthQueryRecoveryHandle) -> WorthQueryRecoveryEffectAuthority {
    WorthQueryRecoveryEffectAuthority::mint_for_test(
        handle.runtime_authority_for_test(),
        handle.authority_identity_for_test(),
    )
}

#[test]
fn recorded_inverse_handle_denies_compensation_from_its_own_mechanism() {
    let handle = probe_handle(worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_account());
    let authority = authority(&handle);
    let denied = compensate_recovery_handle(handle, &authority)
        .expect_err("recorded inverse is not compensation");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::CompensationNotAdmitted
    );
}

#[test]
fn runtime_alone_handle_denies_reconciliation_from_its_own_authority() {
    let handle = probe_handle(worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_account());
    let authority = authority(&handle);
    let denied = reconcile_recovery_handle(handle, &authority)
        .expect_err("runtime-alone authority is not reconciliation");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::ReconciliationNotAdmitted
    );
}
