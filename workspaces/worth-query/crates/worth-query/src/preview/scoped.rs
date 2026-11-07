use crate::basis::SnapshotLineageClass;
use crate::basis_lifecycle::{basis_lifecycle, ScopedObservationBasis};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::live::LiveQueryPlan;

#[cfg(test)]
use super::preview_live_execution_counters;
#[cfg(test)]
use super::PreviewExecutionError;
use super::{
    admit_preview_live_session_plan_component, PreviewLiveCounters, PreviewLiveError,
    PreviewLiveFailureClass, PreviewLiveSessionPlanBinding, PreviewSessionPlanBinding,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedPreviewSessionPlanBinding {
    scoped_basis: ScopedObservationBasis,
    preview_binding: PreviewSessionPlanBinding,
}

impl ScopedPreviewSessionPlanBinding {
    pub fn scoped_basis(&self) -> &ScopedObservationBasis {
        &self.scoped_basis
    }

    pub fn preview_binding(&self) -> &PreviewSessionPlanBinding {
        &self.preview_binding
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedPreviewLiveSessionPlanBinding {
    scoped_binding: ScopedPreviewSessionPlanBinding,
    preview_live: PreviewLiveSessionPlanBinding,
    scoped_live_identity: WorthQueryEvidenceIdentity,
}

impl ScopedPreviewLiveSessionPlanBinding {
    pub fn scoped_binding(&self) -> &ScopedPreviewSessionPlanBinding {
        &self.scoped_binding
    }

    pub(crate) fn preview_live_component(&self) -> &PreviewLiveSessionPlanBinding {
        &self.preview_live
    }

    pub fn report(&self) -> &super::PreviewLiveAdmissionReport {
        self.preview_live.report()
    }

    pub fn live_plan(&self) -> &LiveQueryPlan {
        self.preview_live.live_plan()
    }

    pub fn scoped_live_digest(&self) -> &str {
        self.scoped_live_identity.as_str()
    }
}

pub(crate) fn admit_scoped_preview_session_plan_binding(
    scoped_basis: ScopedObservationBasis,
    preview_binding: PreviewSessionPlanBinding,
) -> Result<ScopedPreviewSessionPlanBinding, PreviewLiveError> {
    ensure_scoped_preview_basis_coherence(&scoped_basis, &preview_binding)?;
    Ok(ScopedPreviewSessionPlanBinding {
        scoped_basis,
        preview_binding,
    })
}

pub(crate) fn admit_scoped_preview_session_plan_binding_from_preview_binding(
    preview_binding: PreviewSessionPlanBinding,
) -> Result<ScopedPreviewSessionPlanBinding, PreviewLiveError> {
    let scoped_basis = scoped_observation_basis_for_preview_binding(&preview_binding)?;
    admit_scoped_preview_session_plan_binding(scoped_basis, preview_binding)
}

pub(crate) fn admit_scoped_preview_live_session_plan(
    scoped_binding: ScopedPreviewSessionPlanBinding,
    live_plan: LiveQueryPlan,
) -> Result<ScopedPreviewLiveSessionPlanBinding, PreviewLiveError> {
    let preview_live = admit_preview_live_session_plan_component(
        scoped_binding.preview_binding.clone(),
        live_plan,
    )?;
    let scoped_live_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::PreviewBasisAdmission)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "scoped_preview_live_session_plan_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis"),
                scoped_binding.scoped_basis().scoped_basis_digest(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("live"),
                preview_live.report().digest(),
            )
            .seal()
            .map_err(|cause| {
                PreviewLiveError::identity_refused(cause, preview_live.report().counters().clone())
            })?;
    Ok(ScopedPreviewLiveSessionPlanBinding {
        scoped_binding,
        preview_live,
        scoped_live_identity,
    })
}

#[cfg(test)]
pub(crate) fn execute_scoped_preview_live_session_plan(
    preview_live: &ScopedPreviewLiveSessionPlanBinding,
) -> Result<super::PreviewLiveExecutionEnvelope, PreviewExecutionError> {
    let envelope = super::PreviewLiveExecutionEnvelope {
        preview_live: preview_live.clone(),
        counters: preview_live_execution_counters(preview_live.preview_live_component())?,
    };
    envelope.check_invariants()?;
    Ok(envelope)
}

pub(crate) fn scoped_observation_basis_for_preview_binding(
    preview_binding: &PreviewSessionPlanBinding,
) -> Result<ScopedObservationBasis, PreviewLiveError> {
    let declaration = match preview_binding
        .preflight()
        .basis()
        .identity()
        .lineage_class()
    {
        SnapshotLineageClass::CurrentHead => basis_lifecycle().current_head(),
        SnapshotLineageClass::ReplayEquivalent | SnapshotLineageClass::FutureExtension => {
            basis_lifecycle().runtime_snapshot(
                preview_binding
                    .preflight()
                    .basis()
                    .identity()
                    .snapshot_identity()
                    .as_str()
                    .to_string(),
                preview_binding
                    .preflight()
                    .basis()
                    .proof()
                    .digest()
                    .as_str(),
            )
        }
    };

    declaration.observe().map_err(|_| {
        scoped_basis_mismatch(
            "preview binding basis lineage could not be admitted as scoped observation proof",
        )
    })
}

fn ensure_scoped_preview_basis_coherence(
    scoped_basis: &ScopedObservationBasis,
    preview_binding: &PreviewSessionPlanBinding,
) -> Result<(), PreviewLiveError> {
    let expected = scoped_observation_basis_for_preview_binding(preview_binding)?;
    if scoped_basis != &expected {
        return Err(scoped_basis_mismatch(
            "scoped preview binding requires exact structural basis parity with the preview preflight lineage",
        ));
    }

    Ok(())
}

fn scoped_basis_mismatch(message: &'static str) -> PreviewLiveError {
    PreviewLiveError {
        failure_class: PreviewLiveFailureClass::PreviewLiveScopedBasisMismatch,
        message,
        counters: PreviewLiveCounters {
            preview_live_broad_fallback_denial_count: 1,
            ..PreviewLiveCounters::default()
        },
    }
}
