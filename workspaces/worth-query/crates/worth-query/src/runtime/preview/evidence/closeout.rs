use super::super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPreviewCloseoutKind {
    Discarded,
    Promoted,
}

impl WorthQueryPreviewCloseoutKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Discarded => "discarded",
            Self::Promoted => "promoted",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPreviewResidueClass {
    SubscriptionState,
    DerivedRuntimeState,
    EffectDeliveryState,
    PendingWriteIntent,
    PreviewWriteStaging,
    TemporalWakeState,
    AsyncResultState,
    MixedCauseState,
    CrossedAuthoritativeResidue,
    AuthoritativeResidue,
}

impl WorthQueryPreviewResidueClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SubscriptionState => "subscription-state",
            Self::DerivedRuntimeState => "derived-runtime-state",
            Self::EffectDeliveryState => "effect-delivery-state",
            Self::PendingWriteIntent => "pending-write-intent",
            Self::PreviewWriteStaging => "preview-write-staging",
            Self::TemporalWakeState => "temporal-wake-state",
            Self::AsyncResultState => "async-result-state",
            Self::MixedCauseState => "mixed-cause-state",
            Self::CrossedAuthoritativeResidue => "crossed-authoritative-residue",
            Self::AuthoritativeResidue => "authoritative-residue",
        }
    }

    pub fn is_authoritative(self) -> bool {
        matches!(
            self,
            Self::CrossedAuthoritativeResidue | Self::AuthoritativeResidue
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPreviewCloseoutEvidence {
    session_label: WorthQuerySessionLabel,
    kind: WorthQueryPreviewCloseoutKind,
    effect_policy: WorthQueryEffectPolicy,
    basis_evidence: Vec<String>,
    basis_admission_identity: WorthQueryEvidenceIdentity,
    preview_basis_snapshot_identity: WorthQuerySnapshotIdentity,
    target_basis_snapshot_identity: WorthQuerySnapshotIdentity,
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
    rebinding_identity: Option<WorthQueryEvidenceIdentity>,
    closeout_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryPreviewCloseoutEvidence {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::runtime::preview) fn new(
        kind: WorthQueryPreviewCloseoutKind,
        effect_policy: WorthQueryEffectPolicy,
        basis_admission: &WorthQueryPreviewBasisAdmission,
        preview_basis_snapshot_identity: &WorthQuerySnapshotIdentity,
        target_basis_snapshot_identity: &WorthQuerySnapshotIdentity,
        preview_binding_count: usize,
        live_binding_count: usize,
        computed_binding_count: usize,
        effect_binding_count: usize,
        subscription_residue_count: usize,
        derived_runtime_residue_count: usize,
        preview_write_staging_count: usize,
        promoted_write_count: usize,
        temporal_wake_residue_count: usize,
        async_result_residue_count: usize,
        mixed_cause_residue_count: usize,
        crossed_authoritative_residue_count: usize,
        effect_delivery_residue_count: usize,
        pending_write_intent_residue_count: usize,
        authoritative_residue_count: usize,
        rebinding_identity: Option<WorthQueryEvidenceIdentity>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let basis_evidence_rows = basis_admission.evidence_rows();
        let mut closeout_builder =
            worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewCloseoutEvidence)
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
                .field_shape(
                    WorthQueryEvidenceTag::new("authority_lane"),
                    basis_admission.authority_lane().as_str(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("basis_evidence_row"),
                    basis_evidence_rows
                        .iter()
                        .map(|row| row.row_digest().as_str()),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("preview_basis_snapshot_identity"),
                    &preview_basis_snapshot_identity.evidence_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("target_basis_snapshot_identity"),
                    &target_basis_snapshot_identity.evidence_identity(),
                )
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
                );
        if let Some(rebinding_identity) = rebinding_identity.as_ref() {
            closeout_builder = closeout_builder.field_value(
                WorthQueryEvidenceTag::new("rebinding_digest"),
                rebinding_identity.as_str(),
            );
        }
        let closeout_identity = closeout_builder.seal()?;
        let basis_evidence = basis_admission.evidence();
        let basis_admission_identity = basis_admission.admission_identity().clone();
        Ok(Self {
            session_label: basis_admission.session_label().clone(),
            kind,
            effect_policy,
            basis_evidence,
            basis_admission_identity,
            preview_basis_snapshot_identity: preview_basis_snapshot_identity.clone(),
            target_basis_snapshot_identity: target_basis_snapshot_identity.clone(),
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
            rebinding_identity,
            closeout_identity,
        })
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

    pub fn kind(&self) -> WorthQueryPreviewCloseoutKind {
        self.kind
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }

    pub fn basis_evidence(&self) -> &[String] {
        &self.basis_evidence
    }

    pub fn basis_admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_admission_identity
    }

    pub fn preview_basis_snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.preview_basis_snapshot_identity
    }

    pub fn target_basis_snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.target_basis_snapshot_identity
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

    pub fn rebinding_digest(&self) -> Option<&str> {
        self.rebinding_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn rebinding_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.rebinding_identity.as_ref()
    }

    pub fn class_count(&self, residue_class: WorthQueryPreviewResidueClass) -> usize {
        match residue_class {
            WorthQueryPreviewResidueClass::SubscriptionState => self.subscription_residue_count,
            WorthQueryPreviewResidueClass::DerivedRuntimeState => {
                self.derived_runtime_residue_count
            }
            WorthQueryPreviewResidueClass::EffectDeliveryState => {
                self.effect_delivery_residue_count
            }
            WorthQueryPreviewResidueClass::PendingWriteIntent => {
                self.pending_write_intent_residue_count
            }
            WorthQueryPreviewResidueClass::PreviewWriteStaging => self.preview_write_staging_count,
            WorthQueryPreviewResidueClass::TemporalWakeState => self.temporal_wake_residue_count,
            WorthQueryPreviewResidueClass::AsyncResultState => self.async_result_residue_count,
            WorthQueryPreviewResidueClass::MixedCauseState => self.mixed_cause_residue_count,
            WorthQueryPreviewResidueClass::CrossedAuthoritativeResidue => {
                self.crossed_authoritative_residue_count
            }
            WorthQueryPreviewResidueClass::AuthoritativeResidue => self.authoritative_residue_count,
        }
    }

    pub fn closeout_digest(&self) -> &str {
        self.closeout_identity.as_str()
    }

    pub fn closeout_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.closeout_identity
    }
}
