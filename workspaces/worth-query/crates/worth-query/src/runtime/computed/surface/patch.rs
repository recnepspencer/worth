use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryDerivedPatch {
    view_name: String,
    commit_identity: WorthQueryCommitIdentity,
    authority_lane: WorthQueryAuthorityLane,
    entity_identity: Option<WorthQueryEntityIdentity>,
    aspect_touches: Vec<WorthQueryAspectTouch>,
    family: WorthQueryDerivedPatchFamily,
    payload: WorthQueryDerivedPatchPayload,
    reason: Option<String>,
}

impl WorthQueryDerivedPatch {
    pub fn incremental(
        view_name: impl Into<String>,
        commit_identity: WorthQueryCommitIdentity,
        entity_identity: WorthQueryEntityIdentity,
        aspect_touches: impl IntoIterator<Item = WorthQueryAspectTouch>,
        payload: WorthQueryDerivedPatchPayload,
    ) -> Self {
        Self {
            view_name: view_name.into(),
            commit_identity,
            authority_lane: WorthQueryAuthorityLane::DerivedRuntimeState,
            entity_identity: Some(entity_identity),
            aspect_touches: aspect_touches.into_iter().collect(),
            family: WorthQueryDerivedPatchFamily::Incremental,
            payload,
            reason: None,
        }
    }

    pub fn whole_refresh_fallback(
        view_name: impl Into<String>,
        commit_identity: WorthQueryCommitIdentity,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            view_name: view_name.into(),
            commit_identity,
            authority_lane: WorthQueryAuthorityLane::DerivedRuntimeState,
            entity_identity: None,
            aspect_touches: Vec::new(),
            family: WorthQueryDerivedPatchFamily::RefreshFallback,
            payload: WorthQueryDerivedPatchPayload::empty_refresh_fallback(),
            reason: Some(reason.into()),
        }
    }

    pub fn whole_refresh_materialized(
        view_name: impl Into<String>,
        commit_identity: WorthQueryCommitIdentity,
        aspect_touches: impl IntoIterator<Item = WorthQueryAspectTouch>,
        payload: WorthQueryDerivedPatchPayload,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            view_name: view_name.into(),
            commit_identity,
            authority_lane: WorthQueryAuthorityLane::DerivedRuntimeState,
            entity_identity: None,
            aspect_touches: aspect_touches.into_iter().collect(),
            family: WorthQueryDerivedPatchFamily::RefreshFallback,
            payload,
            reason: Some(reason.into()),
        }
    }

    pub fn note(&self) -> String {
        match self.family {
            WorthQueryDerivedPatchFamily::Incremental => format!(
                "incremental:{}:{}",
                self.commit_identity
                    .evidence_identity()
                    .reporting_projection(),
                self.entity_identity
                    .as_ref()
                    .map(|identity| identity
                        .evidence_identity()
                        .reporting_projection()
                        .to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ),
            WorthQueryDerivedPatchFamily::RefreshFallback => format!(
                "whole-refresh-fallback:{}:{}",
                self.commit_identity
                    .evidence_identity()
                    .reporting_projection(),
                self.reason.as_deref().unwrap_or("unspecified")
            ),
        }
    }

    pub fn is_refresh_fallback(&self) -> bool {
        self.family == WorthQueryDerivedPatchFamily::RefreshFallback
    }

    #[cfg(test)]
    pub fn retained_payload_rows(&self) -> &[WorthQueryRetainedMaterializedRow] {
        self.payload.retained_rows()
    }

    pub fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn commit_identity(&self) -> &WorthQueryCommitIdentity {
        &self.commit_identity
    }

    pub fn aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.aspect_touches
    }

    pub(super) fn entity_identity(&self) -> Option<&WorthQueryEntityIdentity> {
        self.entity_identity.as_ref()
    }

    pub(super) fn family(&self) -> &WorthQueryDerivedPatchFamily {
        &self.family
    }

    pub(super) fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub(in crate::runtime) fn bind_commit_identity(
        &mut self,
        commit_identity: WorthQueryCommitIdentity,
    ) {
        self.commit_identity = commit_identity;
    }

    pub(in crate::runtime) fn to_mutation_delta(
        &self,
        upstream_view: &str,
    ) -> Result<WorthQueryMutationDelta, IdentityDerivationDenial> {
        Ok(WorthQueryMutationDelta::from_touched_aspects(
            format!("derived:{upstream_view}"),
            self.entity_identity.clone().unwrap_or_else(|| {
                crate::memory_workspace::admit_authored_entity_label(upstream_view)
            }),
            WorthQueryMutationKind::Updated,
            self.aspect_touches.clone(),
        )?)
    }
}
