use super::super::super::{
    WorthQueryAuthorityLane, WorthQueryEffectPolicy, WorthQueryPreviewCloseoutEvidence,
    WorthQueryPreviewCloseoutKind, WorthQueryPreviewOutcome,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::session_label::WorthQuerySessionLabel;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPreviewOutcomeInspection {
    label: WorthQuerySessionLabel,
    closeout_kind: WorthQueryPreviewCloseoutKind,
    effect_policy: WorthQueryEffectPolicy,
    promoted: bool,
    discarded: bool,
    write_count: usize,
    preview_binding_count: usize,
    live_binding_count: usize,
    computed_binding_count: usize,
    effect_binding_count: usize,
    subscription_residue_count: usize,
    derived_runtime_residue_count: usize,
    effect_delivery_residue_count: usize,
    pending_write_intent_residue_count: usize,
    preview_write_staging_count: usize,
    promoted_write_count: usize,
    temporal_wake_residue_count: usize,
    async_result_residue_count: usize,
    mixed_cause_residue_count: usize,
    crossed_authoritative_residue_count: usize,
    authoritative_residue_count: usize,
    source_lane: WorthQueryAuthorityLane,
    target_lane: WorthQueryAuthorityLane,
    basis_evidence: Vec<String>,
    basis_identity: WorthQueryEvidenceIdentity,
    preview_basis_snapshot_identity: WorthQuerySnapshotIdentity,
    target_basis_snapshot_identity: WorthQuerySnapshotIdentity,
    closeout_identity: WorthQueryEvidenceIdentity,
    residue_identity: WorthQueryEvidenceIdentity,
    rebinding_identity: Option<WorthQueryEvidenceIdentity>,
    inspection_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryPreviewOutcomeInspection {
    pub(in crate::runtime) fn from_outcome(
        outcome: &WorthQueryPreviewOutcome,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::from_closeout(
            outcome,
            outcome.closeout_evidence(),
            outcome.promoted(),
            outcome.discarded(),
            outcome.write_count(),
            outcome.preview_binding_count(),
            outcome.closeout_evidence().live_binding_count(),
            outcome.closeout_evidence().computed_binding_count(),
            outcome.effect_binding_count(),
            outcome.closeout_evidence().subscription_residue_count(),
            outcome.closeout_evidence().derived_runtime_residue_count(),
            outcome.effect_delivery_residue_count(),
            outcome.pending_write_intent_residue_count(),
            outcome.closeout_evidence().preview_write_staging_count(),
            outcome.closeout_evidence().promoted_write_count(),
            outcome.closeout_evidence().temporal_wake_residue_count(),
            outcome.closeout_evidence().async_result_residue_count(),
            outcome.closeout_evidence().mixed_cause_residue_count(),
            outcome
                .closeout_evidence()
                .crossed_authoritative_residue_count(),
            outcome.authoritative_residue_count(),
            outcome.source_lane(),
            outcome.target_lane(),
        )?)
    }

    #[allow(clippy::too_many_arguments)]
    fn from_closeout(
        outcome: &WorthQueryPreviewOutcome,
        closeout: &WorthQueryPreviewCloseoutEvidence,
        promoted: bool,
        discarded: bool,
        write_count: usize,
        preview_binding_count: usize,
        live_binding_count: usize,
        computed_binding_count: usize,
        effect_binding_count: usize,
        subscription_residue_count: usize,
        derived_runtime_residue_count: usize,
        effect_delivery_residue_count: usize,
        pending_write_intent_residue_count: usize,
        preview_write_staging_count: usize,
        promoted_write_count: usize,
        temporal_wake_residue_count: usize,
        async_result_residue_count: usize,
        mixed_cause_residue_count: usize,
        crossed_authoritative_residue_count: usize,
        authoritative_residue_count: usize,
        source_lane: WorthQueryAuthorityLane,
        target_lane: WorthQueryAuthorityLane,
    ) -> Result<Self, IdentityDerivationDenial> {
        let basis_evidence = closeout.basis_evidence().to_vec();
        let basis_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::PreviewOutcomeInspectionArtifact,
        )
        .field_shape(WorthQueryEvidenceTag::new("artifact_kind"), "basis")
        .field_value(
            WorthQueryEvidenceTag::new("session_label_identity"),
            outcome.session_label().identity_digest().as_str(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("basis_evidence"),
            basis_evidence.iter().map(String::as_str),
        )
        .seal()?;
        let residue_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::PreviewOutcomeInspectionArtifact,
        )
        .field_shape(WorthQueryEvidenceTag::new("artifact_kind"), "residue")
        .field_value(
            WorthQueryEvidenceTag::new("session_label_identity"),
            outcome.session_label().identity_digest().as_str(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("subscription_residue_count"),
            subscription_residue_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("derived_runtime_residue_count"),
            derived_runtime_residue_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("effect_delivery_residue_count"),
            effect_delivery_residue_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("pending_write_intent_residue_count"),
            pending_write_intent_residue_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("preview_write_staging_count"),
            preview_write_staging_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("promoted_write_count"),
            promoted_write_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("temporal_wake_residue_count"),
            temporal_wake_residue_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("async_result_residue_count"),
            async_result_residue_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("mixed_cause_residue_count"),
            mixed_cause_residue_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("crossed_authoritative_residue_count"),
            crossed_authoritative_residue_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("authoritative_residue_count"),
            authoritative_residue_count,
        )
        .seal()?;
        let inspection_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::PreviewOutcomeInspectionArtifact,
        )
        .field_shape(WorthQueryEvidenceTag::new("artifact_kind"), "inspection")
        .field_value(
            WorthQueryEvidenceTag::new("session_label_identity"),
            outcome.session_label().identity_digest().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("closeout_kind"),
            closeout.kind().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("effect_policy"),
            outcome.effect_policy().as_str(),
        )
        .field_bool(WorthQueryEvidenceTag::new("promoted"), promoted)
        .field_bool(WorthQueryEvidenceTag::new("discarded"), discarded)
        .field_usize(WorthQueryEvidenceTag::new("write_count"), write_count)
        .field_usize(
            WorthQueryEvidenceTag::new("preview_binding_count"),
            preview_binding_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("live_binding_count"),
            live_binding_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("computed_binding_count"),
            computed_binding_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("effect_binding_count"),
            effect_binding_count,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_lane"),
            source_lane.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("target_lane"),
            target_lane.as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("basis_identity"),
            &basis_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("preview_basis_snapshot_identity"),
            &closeout
                .preview_basis_snapshot_identity()
                .evidence_identity()?,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("target_basis_snapshot_identity"),
            &closeout
                .target_basis_snapshot_identity()
                .evidence_identity()?,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("closeout_identity"),
            closeout.closeout_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("residue_identity"),
            &residue_identity,
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("rebinding_identity"),
            closeout.rebinding_identity(),
        )
        .seal()?;

        Ok(Self {
            label: outcome.session_label().clone(),
            closeout_kind: closeout.kind(),
            effect_policy: outcome.effect_policy(),
            promoted,
            discarded,
            write_count,
            preview_binding_count,
            live_binding_count,
            computed_binding_count,
            effect_binding_count,
            subscription_residue_count,
            derived_runtime_residue_count,
            effect_delivery_residue_count,
            pending_write_intent_residue_count,
            preview_write_staging_count,
            promoted_write_count,
            temporal_wake_residue_count,
            async_result_residue_count,
            mixed_cause_residue_count,
            crossed_authoritative_residue_count,
            authoritative_residue_count,
            source_lane,
            target_lane,
            basis_evidence,
            basis_identity,
            preview_basis_snapshot_identity: closeout.preview_basis_snapshot_identity().clone(),
            target_basis_snapshot_identity: closeout.target_basis_snapshot_identity().clone(),
            closeout_identity: closeout.closeout_identity().clone(),
            residue_identity,
            rebinding_identity: closeout.rebinding_identity().cloned(),
            inspection_identity,
        })
    }

    pub fn label(&self) -> &str {
        self.label.display()
    }
    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        &self.label
    }
    pub fn closeout_kind(&self) -> WorthQueryPreviewCloseoutKind {
        self.closeout_kind
    }
    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }
    pub fn promoted(&self) -> bool {
        self.promoted
    }
    pub fn discarded(&self) -> bool {
        self.discarded
    }
    pub fn write_count(&self) -> usize {
        self.write_count
    }
    pub fn preview_binding_count(&self) -> usize {
        self.preview_binding_count
    }
    pub fn live_binding_count(&self) -> usize {
        self.live_binding_count
    }
    pub fn computed_binding_count(&self) -> usize {
        self.computed_binding_count
    }
    pub fn effect_binding_count(&self) -> usize {
        self.effect_binding_count
    }
    pub fn subscription_residue_count(&self) -> usize {
        self.subscription_residue_count
    }
    pub fn derived_runtime_residue_count(&self) -> usize {
        self.derived_runtime_residue_count
    }
    pub fn effect_delivery_residue_count(&self) -> usize {
        self.effect_delivery_residue_count
    }
    pub fn pending_write_intent_residue_count(&self) -> usize {
        self.pending_write_intent_residue_count
    }
    pub fn preview_write_staging_count(&self) -> usize {
        self.preview_write_staging_count
    }
    pub fn promoted_write_count(&self) -> usize {
        self.promoted_write_count
    }
    pub fn temporal_wake_residue_count(&self) -> usize {
        self.temporal_wake_residue_count
    }
    pub fn async_result_residue_count(&self) -> usize {
        self.async_result_residue_count
    }
    pub fn mixed_cause_residue_count(&self) -> usize {
        self.mixed_cause_residue_count
    }
    pub fn crossed_authoritative_residue_count(&self) -> usize {
        self.crossed_authoritative_residue_count
    }
    pub fn authoritative_residue_count(&self) -> usize {
        self.authoritative_residue_count
    }
    pub fn source_lane(&self) -> WorthQueryAuthorityLane {
        self.source_lane
    }
    pub fn target_lane(&self) -> WorthQueryAuthorityLane {
        self.target_lane
    }
    pub fn basis_evidence(&self) -> &[String] {
        &self.basis_evidence
    }
    pub fn basis_digest(&self) -> &str {
        self.basis_identity.as_str()
    }
    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }
    pub fn preview_basis_snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.preview_basis_snapshot_identity
    }
    pub fn closeout_digest(&self) -> &str {
        self.closeout_identity.as_str()
    }
    pub fn target_basis_snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.target_basis_snapshot_identity
    }
    pub fn closeout_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.closeout_identity
    }
    pub fn residue_digest(&self) -> &str {
        self.residue_identity.as_str()
    }
    pub fn residue_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.residue_identity
    }
    pub fn rebinding_digest(&self) -> Option<&str> {
        self.rebinding_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }
    pub fn rebinding_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.rebinding_identity.as_ref()
    }
    pub fn inspection_digest(&self) -> &str {
        self.inspection_identity.as_str()
    }
    pub fn inspection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inspection_identity
    }
}
