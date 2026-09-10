//! Handle-owned mechanism and authority-axis evidence.

use worth_query_installation::facade::WorthQueryInstalledAftermathContract;

use super::{
    compensate_recovery_handle, reconcile_recovery_handle, WorthQueryRecoveryEffectAuthority,
};
use crate::domain_computation::application_aftermath::aftermath_schema_fixture as fixture;
use crate::domain_computation::application_aftermath::recovery_handle::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleBindingAxisProbe,
    WorthQueryRecoveryHandleDenialKind,
};

fn probe_handle(aftermath: WorthQueryInstalledAftermathContract) -> WorthQueryRecoveryHandle {
    let binding = WorthQueryRecoveryHandleBindingAxisProbe::real()
        .installed_aftermath(aftermath)
        .expires_at_unix_ms(Some(u64::MAX))
        .finish();
    WorthQueryRecoveryHandle::axis_probe(binding)
}

fn authority(handle: &WorthQueryRecoveryHandle) -> WorthQueryRecoveryEffectAuthority {
    WorthQueryRecoveryEffectAuthority::mint(handle.runtime_authority(), handle.authority_identity())
}

#[test]
fn recorded_inverse_handle_denies_compensation_from_its_own_mechanism() {
    let handle = probe_handle(fixture::freeze_account());
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
    let handle = probe_handle(fixture::freeze_account());
    let authority = authority(&handle);
    let denied = reconcile_recovery_handle(handle, &authority)
        .expect_err("runtime-alone authority is not reconciliation");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::ReconciliationNotAdmitted
    );
}
