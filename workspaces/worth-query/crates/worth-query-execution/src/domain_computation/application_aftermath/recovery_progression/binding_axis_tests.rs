//! Per-axis binding drift with a production-derived positive twin (R8.28).

use worth_relational::facade::history::BranchId;

use super::WorthQueryRecoveryBindingCurrentTruth;
use crate::domain_computation::application_aftermath::recovery_handle::{
    WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleBindingAxisProbe,
    WorthQueryRecoveryHandleDenialKind,
};
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;

fn assert_axis_drift(
    kind: WorthQueryRecoveryHandleDenialKind,
    mutate: impl FnOnce(
        WorthQueryRecoveryHandleBindingAxisProbe,
        &WorthQueryRecoveryHandleBinding,
    ) -> WorthQueryRecoveryHandleBindingAxisProbe,
) {
    let baseline = WorthQueryRecoveryHandleBindingAxisProbe::real().finish();
    let truth = WorthQueryRecoveryBindingCurrentTruth::axis_probe(&baseline);
    truth
        .check(&baseline)
        .expect("production-derived twin admits");

    let drifted = mutate(
        WorthQueryRecoveryHandleBindingAxisProbe::from_binding(baseline.clone()),
        &baseline,
    )
    .finish();
    let denied = truth.check(&drifted).expect_err("drifted binding denies");
    assert_eq!(denied.kind(), kind);
}

#[test]
fn schema_mismatch_drift_denies_distinctly() {
    assert_axis_drift(
        WorthQueryRecoveryHandleDenialKind::SchemaMismatch,
        |probe, baseline| {
            let mut schema = *baseline.schema_identity();
            schema[0] ^= 1;
            probe.schema_identity(schema)
        },
    );
}

#[test]
fn branch_mismatch_drift_denies_distinctly() {
    assert_axis_drift(
        WorthQueryRecoveryHandleDenialKind::BranchMismatch,
        |probe, baseline| {
            probe
                .branch(BranchId("foreign-branch".to_owned()))
                .application_binding_generation(baseline.application_binding_generation() + 1)
        },
    );
}

#[test]
fn application_binding_generation_drift_denies_on_its_own_axis() {
    assert_axis_drift(
        WorthQueryRecoveryHandleDenialKind::ApplicationBindingGenerationMismatch,
        |probe, baseline| {
            probe.application_binding_generation(baseline.application_binding_generation() + 1)
        },
    );
}

#[test]
fn foreign_branch_equal_ordinal_drift_denies_distinctly() {
    assert_axis_drift(
        WorthQueryRecoveryHandleDenialKind::ForeignBranchEqualOrdinal,
        |probe, _| probe.branch(BranchId("foreign-equal-generation".to_owned())),
    );
}

#[test]
fn operation_mismatch_drift_denies_distinctly() {
    assert_axis_drift(
        WorthQueryRecoveryHandleDenialKind::OperationMismatch,
        |probe, baseline| {
            let mut operation = *baseline.installed_operation();
            operation[0] ^= 1;
            probe.installed_operation(operation)
        },
    );
}

#[test]
fn governed_input_mismatch_drift_denies_distinctly() {
    assert_axis_drift(
        WorthQueryRecoveryHandleDenialKind::GovernedInputMismatch,
        |probe, _| probe.retained_governed_input_identity(Some([0x67; 32])),
    );
}

#[test]
fn foreign_principal_drift_denies_distinctly() {
    assert_axis_drift(
        WorthQueryRecoveryHandleDenialKind::ForeignPrincipal,
        |probe, baseline| {
            let current = baseline.principal_scope();
            let principal_scope = WorthQueryOperationScopeBinding::axis_probe_scope(
                current.runtime_authority(),
                current.binding_identity().clone(),
                current.operation_authority_identity(),
                9,
                99,
                2,
                current.scope().partition_id(),
                current.scope().local_slot(),
                current.scope().generation(),
            );
            probe.principal_scope(principal_scope)
        },
    );
}

#[test]
fn installed_aftermath_identity_drift_denies_distinctly() {
    assert_axis_drift(
        WorthQueryRecoveryHandleDenialKind::CompatibilityGenerationMismatch,
        |probe, _| {
            probe.installed_aftermath(
                crate::domain_computation::application_aftermath::aftermath_schema_fixture::transfer(
                ),
            )
        },
    );
}
