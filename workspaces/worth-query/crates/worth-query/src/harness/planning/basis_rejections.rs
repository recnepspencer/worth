use super::{direct_validated_bundle, runtime_basis_intent, runtime_resolved_identity};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::facade::foundation::resolve_snapshot_basis;
use crate::facade::foundation::{
    BasisAuthorityFamily, BasisResolutionError, BasisResolutionMode, ExecutionBasisIntent,
    ResolvedSnapshotIdentity, SnapshotLineageClass,
};
use crate::facade::policy::{plan_validated_bundle, planning_request_context_for_direct};

#[test]
fn plan_and_resolved_basis_preflight_successfully_couple() -> Result<(), IdentityDerivationDenial> {
    let bundle = direct_validated_bundle();
    let request = planning_request_context_for_direct(&bundle, runtime_basis_intent()).unwrap();
    let planned = plan_validated_bundle(&bundle, request).unwrap();
    let basis = resolve_snapshot_basis(
        runtime_basis_intent(),
        runtime_resolved_identity(bundle.query().schema_basis().clone())?,
        BasisResolutionMode::RuntimeDirect,
    )
    .unwrap();

    let preflight = crate::facade::foundation::preflight_execution_basis(planned, basis).unwrap();
    assert_eq!(
        preflight.report().basis_digest(),
        preflight.basis().proof().digest()
    );
    assert_eq!(preflight.report().snapshot_basis_resolution_count(), 1);

    Ok(())
}

#[test]
fn store_backend_planning_is_rejected_until_parity_is_admitted() {
    let bundle = direct_validated_bundle();
    let store_intent = ExecutionBasisIntent::new(
        BasisAuthorityFamily::Store,
        SnapshotLineageClass::CurrentHead,
        false,
    );
    let request = planning_request_context_for_direct(&bundle, store_intent).unwrap();
    let error = plan_validated_bundle(&bundle, request).unwrap_err();
    assert_eq!(
        error,
        crate::facade::policy::PlanningError::UnsupportedBackendParityRequest
    );
}

#[test]
fn resolve_snapshot_basis_rejects_identity_mismatch() -> Result<(), IdentityDerivationDenial> {
    let bundle = direct_validated_bundle();
    let error = resolve_snapshot_basis(
        runtime_basis_intent(),
        ResolvedSnapshotIdentity::new(
            BasisAuthorityFamily::Store,
            Some("workspace-main".to_string()),
            crate::memory_workspace::admit_external_snapshot_label("snapshot-2")?
                .evidence_identity(),
            bundle.query().schema_basis().clone(),
            SnapshotLineageClass::CurrentHead,
        ),
        BasisResolutionMode::StoreDirect,
    )
    .unwrap_err();

    assert_eq!(error, BasisResolutionError::ResolutionIdentityMismatch);

    Ok(())
}

#[test]
fn fallback_admission_is_rejected_until_supported_shape_exists() {
    let bundle = direct_validated_bundle();
    let request = planning_request_context_for_direct(
        &bundle,
        ExecutionBasisIntent::new(
            BasisAuthorityFamily::Runtime,
            SnapshotLineageClass::CurrentHead,
            true,
        ),
    )
    .unwrap();
    let error = plan_validated_bundle(&bundle, request).unwrap_err();
    assert_eq!(
        error,
        crate::facade::policy::PlanningError::UnsupportedFallbackShape
    );
}
