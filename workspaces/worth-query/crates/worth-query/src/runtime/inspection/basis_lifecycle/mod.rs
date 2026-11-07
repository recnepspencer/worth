use crate::application::WorthQueryAdmittedWorldBasis;
use crate::basis_lifecycle::{
    BasisFamily, BasisLifecyclePosture, ScopedBasisProof, ScopedInspectionBasis,
    ScopedMaterializationBasis, ScopedMutationPreparationBasis, ScopedObservationBasis,
    ScopedPreviewCloseoutBasis, ScopedReplayBasis, ScopedSubscriptionActivationBasis,
    ScopedSubscriptionDeclarationBasis,
};
use crate::identity::hash_parts;
use crate::runtime::evidence_identities::{
    runtime_state_snapshot_basis_label_identity, runtime_state_snapshot_result_shape_label_identity,
};
use crate::runtime::{WorthQueryAuthorityLane, WorthQueryRuntimeStateKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBasisLifecycleInspection {
    subject_label: &'static str,
    state_kind: WorthQueryRuntimeStateKind,
    authority_lane: WorthQueryAuthorityLane,
    basis_digest: String,
    shape_digest: String,
    family: Option<BasisFamily>,
    operation_lane: Option<&'static str>,
    lifecycle_posture: Option<BasisLifecyclePosture>,
    lower_runtime_binding_digest: Option<String>,
    support_digest: Option<String>,
    explanation: String,
    inspection_digest: String,
}

impl WorthQueryBasisLifecycleInspection {
    pub(in crate::runtime) fn from_admitted_world_basis(
        basis: &WorthQueryAdmittedWorldBasis,
    ) -> Self {
        let basis_digest =
            runtime_state_snapshot_basis_label_identity(basis.basis_lifecycle_support_identity())
                .as_str()
                .to_string();
        let shape_digest =
            runtime_state_snapshot_result_shape_label_identity(basis.handle_identity())
                .as_str()
                .to_string();
        Self::assemble(
            "admitted_world_basis",
            WorthQueryAuthorityLane::AuthoritativeTruth,
            basis_digest,
            shape_digest,
            None,
            None,
            None,
            None,
            Some(basis.support_snapshot_digest().to_string()),
            format!(
                "retained admitted world basis `{}` exposes canonical basis lifecycle support `{}`",
                basis.domain_key(),
                basis.basis_lifecycle_support_for_reporting()
            ),
        )
    }

    fn from_scoped(
        basis: &impl ScopedBasisProof,
        subject_label: &'static str,
        operation_lane: &'static str,
    ) -> Self {
        Self::assemble(
            subject_label,
            authority_lane_for_family(basis.family()),
            basis.scoped_basis_digest().to_string(),
            basis.capability_digest().to_string(),
            Some(basis.family()),
            Some(operation_lane),
            Some(basis.lifecycle()),
            basis
                .expected_lower_runtime_binding_digest()
                .map(str::to_string),
            None,
            format!(
                "{subject_label} carries sealed `{}` authority with `{}` lifecycle posture",
                basis.authority().as_str(),
                basis.lifecycle().as_str()
            ),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn assemble(
        subject_label: &'static str,
        authority_lane: WorthQueryAuthorityLane,
        basis_digest: String,
        shape_digest: String,
        family: Option<BasisFamily>,
        operation_lane: Option<&'static str>,
        lifecycle_posture: Option<BasisLifecyclePosture>,
        lower_runtime_binding_digest: Option<String>,
        support_digest: Option<String>,
        explanation: String,
    ) -> Self {
        let inspection_digest = hash_parts(&[
            "worth_query_basis_lifecycle_inspection_v2".to_string(),
            format!("subject:{subject_label}"),
            format!("authority_lane:{}", authority_lane.as_str()),
            format!("basis:{basis_digest}"),
            format!("shape:{shape_digest}"),
            format!(
                "family:{}",
                family.map(|value| value.as_str()).unwrap_or("none")
            ),
            format!("lane:{}", operation_lane.unwrap_or("none")),
        ]);
        Self {
            subject_label,
            state_kind: WorthQueryRuntimeStateKind::Ready,
            authority_lane,
            basis_digest,
            shape_digest,
            family,
            operation_lane,
            lifecycle_posture,
            lower_runtime_binding_digest,
            support_digest,
            explanation,
            inspection_digest,
        }
    }

    pub fn subject_label(&self) -> &'static str {
        self.subject_label
    }
    pub fn state_kind(&self) -> WorthQueryRuntimeStateKind {
        self.state_kind
    }
    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }
    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }
    pub fn shape_digest(&self) -> &str {
        &self.shape_digest
    }
    pub fn family(&self) -> Option<BasisFamily> {
        self.family
    }
    pub fn operation_lane(&self) -> Option<&'static str> {
        self.operation_lane
    }
    pub fn lifecycle_posture(&self) -> Option<BasisLifecyclePosture> {
        self.lifecycle_posture
    }
    pub fn lower_runtime_binding_digest(&self) -> Option<&str> {
        self.lower_runtime_binding_digest.as_deref()
    }
    pub fn support_digest(&self) -> Option<&str> {
        self.support_digest.as_deref()
    }
    pub fn explanation(&self) -> &str {
        &self.explanation
    }
    pub fn inspection_digest(&self) -> &str {
        &self.inspection_digest
    }
}

macro_rules! impl_scoped_inspection {
    ($target:ty, $label:literal, $lane:literal) => {
        impl From<&$target> for WorthQueryBasisLifecycleInspection {
            fn from(value: &$target) -> Self {
                Self::from_scoped(value, $label, $lane)
            }
        }
    };
}

impl_scoped_inspection!(
    ScopedObservationBasis,
    "scoped_observation_basis",
    "observation"
);
impl_scoped_inspection!(
    ScopedMutationPreparationBasis,
    "scoped_mutation_preparation_basis",
    "mutation_preparation"
);
impl_scoped_inspection!(ScopedReplayBasis, "scoped_replay_basis", "replay");
impl_scoped_inspection!(
    ScopedInspectionBasis,
    "scoped_inspection_basis",
    "inspection"
);
impl_scoped_inspection!(
    ScopedMaterializationBasis,
    "scoped_materialization_basis",
    "materialization"
);
impl_scoped_inspection!(
    ScopedSubscriptionDeclarationBasis,
    "scoped_subscription_declaration_basis",
    "subscription_declaration"
);
impl_scoped_inspection!(
    ScopedSubscriptionActivationBasis,
    "scoped_subscription_activation_basis",
    "subscription_activation"
);
impl_scoped_inspection!(
    ScopedPreviewCloseoutBasis,
    "scoped_preview_closeout_basis",
    "preview_closeout"
);

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
