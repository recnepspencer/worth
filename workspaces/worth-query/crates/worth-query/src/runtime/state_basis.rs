use super::evidence_identities::{
    runtime_state_snapshot_basis_label_identity, runtime_state_snapshot_result_shape_label_identity,
};
use super::state::WorthQueryRuntimeStateTarget;
use super::{
    WorthQueryAuthorityLane, WorthQueryRuntime, WorthQueryRuntimeError,
    WorthQueryRuntimeStateSnapshot,
};
use crate::application::WorthQueryAdmittedWorldBasis;
use crate::basis_lifecycle::{
    BasisFamily, ScopedBasisProof, ScopedInspectionBasis, ScopedMaterializationBasis,
    ScopedMutationPreparationBasis, ScopedObservationBasis, ScopedPreviewCloseoutBasis,
    ScopedReplayBasis, ScopedSubscriptionActivationBasis, ScopedSubscriptionDeclarationBasis,
};
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

impl WorthQueryRuntimeStateTarget for &WorthQueryAdmittedWorldBasis {
    fn into_state_snapshot(
        self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryRuntime,
    ) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
        runtime
            .validate_installed_domain_authority(self.installed_authority())
            .map_err(WorthQueryRuntimeError::InstalledDomainAuthorityDenied)?;
        Ok(WorthQueryRuntimeStateSnapshot::ready(
            runtime_state_snapshot_basis_label_identity(self.basis_lifecycle_support_identity()),
            runtime_state_snapshot_result_shape_label_identity(self.handle_identity()),
            WorthQueryAuthorityLane::AuthoritativeTruth,
            format!(
                "retained admitted world basis `{}` is ready with basis lifecycle support `{}`",
                self.domain_key(),
                self.basis_lifecycle_support_for_reporting()
            ),
        ))
    }
}

macro_rules! impl_scoped_basis_state_target {
    ($target:ty, $label:literal) => {
        impl WorthQueryRuntimeStateTarget for &$target {
            fn into_state_snapshot(
                self,
                _execution: &worth_execution::ExecutionRequest<'_, '_>,
                _runtime: &WorthQueryRuntime,
            ) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
                Ok(scoped_basis_snapshot(self, $label))
            }
        }
    };
}

impl_scoped_basis_state_target!(ScopedObservationBasis, "scoped observation basis");
impl_scoped_basis_state_target!(
    ScopedMutationPreparationBasis,
    "scoped mutation-preparation basis"
);
impl_scoped_basis_state_target!(ScopedReplayBasis, "scoped replay basis");
impl_scoped_basis_state_target!(ScopedInspectionBasis, "scoped inspection basis");
impl_scoped_basis_state_target!(ScopedMaterializationBasis, "scoped materialization basis");
impl_scoped_basis_state_target!(
    ScopedSubscriptionDeclarationBasis,
    "scoped subscription-declaration basis"
);
impl_scoped_basis_state_target!(
    ScopedSubscriptionActivationBasis,
    "scoped subscription-activation basis"
);
impl_scoped_basis_state_target!(ScopedPreviewCloseoutBasis, "scoped preview-closeout basis");

fn scoped_basis_snapshot(
    basis: &impl ScopedBasisProof,
    label: &str,
) -> WorthQueryRuntimeStateSnapshot {
    WorthQueryRuntimeStateSnapshot::ready(
        runtime_state_snapshot_basis_label_identity(&scoped_identity(
            "scoped_basis",
            basis.scoped_basis_digest(),
        )),
        runtime_state_snapshot_result_shape_label_identity(&scoped_identity(
            "basis_capability",
            basis.capability_digest(),
        )),
        authority_lane_for_family(basis.family()),
        format!(
            "{label} is ready with `{}` authority and `{}` lifecycle posture",
            basis.authority().as_str(),
            basis.lifecycle().as_str()
        ),
    )
}

fn scoped_identity(family: &'static str, digest: &str) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(WorthQueryEvidenceTag::new("identity_family"), family)
        .field_shape(WorthQueryEvidenceTag::new("canonical_digest"), digest)
        .seal()
}

fn authority_lane_for_family(family: BasisFamily) -> WorthQueryAuthorityLane {
    match family {
        BasisFamily::BranchHead | BasisFamily::BranchSnapshot => {
            WorthQueryAuthorityLane::BranchLocalTruth
        }
        BasisFamily::Preview | BasisFamily::PreviewDerived => WorthQueryAuthorityLane::PreviewTruth,
        BasisFamily::HistoricalSnapshot
        | BasisFamily::HistoricalCommit
        | BasisFamily::StoreBacked
        | BasisFamily::DurableReload => WorthQueryAuthorityLane::BridgeExternalState,
        BasisFamily::CurrentHead
        | BasisFamily::RuntimeSnapshot
        | BasisFamily::TenantScoped
        | BasisFamily::PolicyScoped => WorthQueryAuthorityLane::AuthoritativeTruth,
    }
}
