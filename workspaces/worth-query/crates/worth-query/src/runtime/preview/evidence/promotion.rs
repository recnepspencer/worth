use super::super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPreviewPromotionDenialKind {
    StaleBasis,
    WriteFailed,
    AtomicBatchUnsupported,
    RebindingRequired,
}

impl WorthQueryPreviewPromotionDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StaleBasis => "stale-basis",
            Self::WriteFailed => "write-failed",
            Self::AtomicBatchUnsupported => "atomic-batch-unsupported",
            Self::RebindingRequired => "rebinding-required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPreviewPromotionDenialEvidence {
    session_label: WorthQuerySessionLabel,
    kind: WorthQueryPreviewPromotionDenialKind,
    effect_policy: WorthQueryEffectPolicy,
    basis_evidence: Vec<String>,
    basis_snapshot_identity: WorthQuerySnapshotIdentity,
    promotion_snapshot_identity: WorthQuerySnapshotIdentity,
    staged_preview_write_count: usize,
    promoted_write_count: usize,
    failed_write_sequence: Option<usize>,
    preview_binding_count: usize,
    crossed_authoritative_residue_count: usize,
    recovery_posture: String,
    rebinding_identity: Option<WorthQueryEvidenceIdentity>,
    reason: String,
    denial_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryPreviewPromotionDenialEvidence {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::runtime::preview) fn new(
        kind: WorthQueryPreviewPromotionDenialKind,
        effect_policy: WorthQueryEffectPolicy,
        basis_admission: &WorthQueryPreviewBasisAdmission,
        basis_snapshot_identity: &WorthQuerySnapshotIdentity,
        promotion_snapshot_identity: &WorthQuerySnapshotIdentity,
        staged_preview_write_count: usize,
        promoted_write_count: usize,
        failed_write_sequence: Option<usize>,
        preview_binding_count: usize,
        crossed_authoritative_residue_count: usize,
        recovery_posture: String,
        rebinding_identity: Option<WorthQueryEvidenceIdentity>,
        reason: String,
    ) -> Result<Self, IdentityDerivationDenial> {
        let basis_evidence_rows = basis_admission.evidence_rows();
        let mut denial_builder =
            worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewPromotionDenialEvidence)
                .field_value(
                    WorthQueryEvidenceTag::new("session_label_identity"),
                    basis_admission.label_identity().as_str(),
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
                .field_shape(
                    WorthQueryEvidenceTag::new("effect_policy"),
                    effect_policy.as_str(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("basis_admission_digest"),
                    basis_admission.admission_digest().as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("basis_snapshot_identity"),
                    &basis_snapshot_identity.evidence_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("promotion_snapshot_identity"),
                    &promotion_snapshot_identity.evidence_identity(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("basis_evidence_row"),
                    basis_evidence_rows
                        .iter()
                        .map(|row| row.row_digest().as_str()),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("staged_preview_write_count"),
                    staged_preview_write_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("promoted_write_count"),
                    promoted_write_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("preview_binding_count"),
                    preview_binding_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("crossed_authoritative_residue_count"),
                    crossed_authoritative_residue_count,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("recovery_posture"),
                    recovery_posture.as_str(),
                )
                .field_value(WorthQueryEvidenceTag::new("reason"), reason.as_str());
        if let Some(failed_write_sequence) = failed_write_sequence {
            denial_builder = denial_builder.field_usize(
                WorthQueryEvidenceTag::new("failed_write_sequence"),
                failed_write_sequence,
            );
        }
        if let Some(rebinding_identity) = rebinding_identity.as_ref() {
            denial_builder = denial_builder.field_value(
                WorthQueryEvidenceTag::new("rebinding_digest"),
                rebinding_identity.as_str(),
            );
        }
        let denial_identity = denial_builder.seal()?;
        let basis_evidence = basis_admission.evidence();
        Ok(Self {
            session_label: basis_admission.session_label().clone(),
            kind,
            effect_policy,
            basis_evidence,
            basis_snapshot_identity: basis_snapshot_identity.clone(),
            promotion_snapshot_identity: promotion_snapshot_identity.clone(),
            staged_preview_write_count,
            promoted_write_count,
            failed_write_sequence,
            preview_binding_count,
            crossed_authoritative_residue_count,
            recovery_posture,
            rebinding_identity,
            reason,
            denial_identity,
        })
    }

    pub(in crate::runtime::preview) fn stale_basis(
        effect_policy: WorthQueryEffectPolicy,
        basis_admission: &WorthQueryPreviewBasisAdmission,
        basis_snapshot_identity: &WorthQuerySnapshotIdentity,
        promotion_snapshot_identity: &WorthQuerySnapshotIdentity,
        staged_preview_write_count: usize,
        preview_binding_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryPreviewPromotionDenialKind::StaleBasis,
            effect_policy,
            basis_admission,
            basis_snapshot_identity,
            promotion_snapshot_identity,
            staged_preview_write_count,
            0,
            None,
            preview_binding_count,
            0,
            "refresh_preview_basis".to_string(),
            None,
            "preview promotion rejected because authoritative basis changed before promotion"
                .to_string(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::runtime::preview) fn write_failed(
        effect_policy: WorthQueryEffectPolicy,
        basis_admission: &WorthQueryPreviewBasisAdmission,
        basis_snapshot_identity: &WorthQuerySnapshotIdentity,
        promotion_snapshot_identity: &WorthQuerySnapshotIdentity,
        staged_preview_write_count: usize,
        promoted_write_count: usize,
        failed_write_sequence: usize,
        preview_binding_count: usize,
        reason: String,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryPreviewPromotionDenialKind::WriteFailed,
            effect_policy,
            basis_admission,
            basis_snapshot_identity,
            promotion_snapshot_identity,
            staged_preview_write_count,
            promoted_write_count,
            Some(failed_write_sequence),
            preview_binding_count,
            0,
            "retry_authoritative_write".to_string(),
            None,
            reason,
        )
    }

    pub(in crate::runtime::preview) fn atomic_batch_unsupported(
        effect_policy: WorthQueryEffectPolicy,
        basis_admission: &WorthQueryPreviewBasisAdmission,
        basis_snapshot_identity: &WorthQuerySnapshotIdentity,
        promotion_snapshot_identity: &WorthQuerySnapshotIdentity,
        staged_preview_write_count: usize,
        preview_binding_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryPreviewPromotionDenialKind::AtomicBatchUnsupported,
            effect_policy,
            basis_admission,
            basis_snapshot_identity,
            promotion_snapshot_identity,
            staged_preview_write_count,
            0,
            None,
            preview_binding_count,
            0,
            "promote_with_atomic_batch_support".to_string(),
            None,
            "preview promotion rejected because multiple staged writes require atomic promotion support"
                .to_string(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::runtime::preview) fn rebinding_required(
        effect_policy: WorthQueryEffectPolicy,
        basis_admission: &WorthQueryPreviewBasisAdmission,
        basis_snapshot_identity: &WorthQuerySnapshotIdentity,
        promotion_snapshot_identity: &WorthQuerySnapshotIdentity,
        staged_preview_write_count: usize,
        preview_binding_count: usize,
        crossed_authoritative_residue_count: usize,
        rebinding_identity: WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryPreviewPromotionDenialKind::RebindingRequired,
            effect_policy,
            basis_admission,
            basis_snapshot_identity,
            promotion_snapshot_identity,
            staged_preview_write_count,
            0,
            None,
            preview_binding_count,
            crossed_authoritative_residue_count,
            "discard_preview_and_readmit_authoritative".to_string(),
            Some(rebinding_identity),
            "preview promotion rejected because preview-owned temporal or async residue requires authoritative re-admission before promotion"
                .to_string(),
        )
    }

    pub fn label(&self) -> &str {
        self.session_label.display()
    }

    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        &self.session_label
    }

    pub fn label_identity(&self) -> &crate::WorthQueryEvidenceIdentity {
        self.session_label.identity_digest()
    }

    pub fn kind(&self) -> WorthQueryPreviewPromotionDenialKind {
        self.kind
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }

    pub fn basis_evidence(&self) -> &[String] {
        &self.basis_evidence
    }

    pub fn basis_snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.basis_snapshot_identity
    }

    pub fn promotion_snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.promotion_snapshot_identity
    }

    pub fn staged_preview_write_count(&self) -> usize {
        self.staged_preview_write_count
    }

    pub fn promoted_write_count(&self) -> usize {
        self.promoted_write_count
    }

    pub fn failed_write_sequence(&self) -> Option<usize> {
        self.failed_write_sequence
    }

    pub fn preview_binding_count(&self) -> usize {
        self.preview_binding_count
    }

    pub fn crossed_authoritative_residue_count(&self) -> usize {
        self.crossed_authoritative_residue_count
    }

    pub fn recovery_posture(&self) -> &str {
        &self.recovery_posture
    }

    pub fn rebinding_digest(&self) -> Option<&str> {
        self.rebinding_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn rebinding_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.rebinding_identity.as_ref()
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn denial_digest(&self) -> &str {
        self.denial_identity.as_str()
    }

    pub fn denial_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.denial_identity
    }
}
