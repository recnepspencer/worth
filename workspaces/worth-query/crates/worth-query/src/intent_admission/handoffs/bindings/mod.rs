use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod inspection;
mod read;
mod routing;
mod unified_inspection;

use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryCommitIdentity;
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryEffectDelivery, WorthQueryEffectWriteAdjacentTrigger,
    WorthQueryIntentDeclaration, WorthQueryIntentSourceLane, WorthQueryWriteCommand,
};

use super::{
    WorthQueryAuthoritativeIntentExecutionHandoff,
    WorthQueryAuthoritativeMutationBatchExecutionHandoff,
    WorthQueryAuthoritativeMutationExecutionHandoff,
    WorthQueryEffectTriggeredIntentExecutionHandoff,
};
use crate::intent_admission::{
    WorthQueryIntentAdmissionCoveredEntrypoint, WorthQueryIntentAdmissionExecutionSeam,
    WorthQueryIntentAdmissionFamily,
};
pub use inspection::{
    WorthQueryDerivedInspectionExecutionBinding, WorthQueryDerivedMaterializationExecutionBinding,
};
pub use read::{WorthQueryLiveReadExecutionBinding, WorthQueryReadExecutionBinding};
pub use routing::WorthQueryExistingTruthProbeExecutionBinding;
pub use unified_inspection::WorthQueryUnifiedInspectionExecutionBinding;

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAuthoritativeIntentExecutionBinding {
    handoff: WorthQueryAuthoritativeIntentExecutionHandoff,
    binding_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryEffectTriggeredIntentExecutionBinding {
    handoff: WorthQueryEffectTriggeredIntentExecutionHandoff,
    effect_name: String,
    trigger_commit_identity: WorthQueryCommitIdentity,
    write_adjacent_trigger: WorthQueryEffectWriteAdjacentTrigger,
    pending_delivery_digest: String,
    binding_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAuthoritativeMutationExecutionBinding {
    handoff: WorthQueryAuthoritativeMutationExecutionHandoff,
    binding_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAuthoritativeMutationBatchExecutionBinding {
    handoff: WorthQueryAuthoritativeMutationBatchExecutionHandoff,
    binding_digest: String,
}

impl WorthQueryAuthoritativeIntentExecutionBinding {
    pub(crate) fn from_handoff(
        handoff: WorthQueryAuthoritativeIntentExecutionHandoff,
    ) -> Result<Self, IdentityDerivationDenial> {
        let binding_digest = intent_execution_binding_identity(
            "authoritative-intent-execution",
            handoff.handoff_digest(),
            None,
        )?;
        Ok(Self {
            handoff,
            binding_digest,
        })
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.handoff.family()
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.handoff.entrypoint()
    }

    pub fn execution_seam(&self) -> WorthQueryIntentAdmissionExecutionSeam {
        self.handoff.execution_seam()
    }

    pub fn declaration(&self) -> &WorthQueryIntentDeclaration {
        self.handoff.declaration()
    }

    pub fn handoff(&self) -> &WorthQueryAuthoritativeIntentExecutionHandoff {
        &self.handoff
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }
}

impl WorthQueryEffectTriggeredIntentExecutionBinding {
    pub(crate) fn from_handoff_and_delivery(
        handoff: WorthQueryEffectTriggeredIntentExecutionHandoff,
        pending_delivery: &WorthQueryEffectDelivery,
    ) -> Result<Self, IdentityDerivationDenial> {
        let pending_delivery_digest = hash_effect_pending_delivery(pending_delivery)?;
        let binding_digest = intent_execution_binding_identity(
            "effect-intent-execution",
            handoff.handoff_digest(),
            Some(&pending_delivery_digest),
        )?;
        Ok(Self {
            handoff,
            effect_name: pending_delivery.effect_name().to_string(),
            trigger_commit_identity: pending_delivery.commit_identity().clone(),
            write_adjacent_trigger: pending_delivery.write_adjacent_trigger().clone(),
            pending_delivery_digest,
            binding_digest,
        })
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.handoff.family()
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.handoff.entrypoint()
    }

    pub fn execution_seam(&self) -> WorthQueryIntentAdmissionExecutionSeam {
        self.handoff.execution_seam()
    }

    pub fn declaration(&self) -> &WorthQueryIntentDeclaration {
        self.handoff.declaration()
    }

    pub fn handoff(&self) -> &WorthQueryEffectTriggeredIntentExecutionHandoff {
        &self.handoff
    }

    pub fn effect_name(&self) -> &str {
        &self.effect_name
    }

    pub fn trigger_commit_identity(&self) -> &WorthQueryCommitIdentity {
        &self.trigger_commit_identity
    }

    pub fn pending_delivery_digest(&self) -> &str {
        &self.pending_delivery_digest
    }

    pub fn write_adjacent_trigger(&self) -> &WorthQueryEffectWriteAdjacentTrigger {
        &self.write_adjacent_trigger
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }

    pub(crate) fn matches_pending_delivery(
        &self,
        pending_delivery: &WorthQueryEffectDelivery,
    ) -> Result<bool, IdentityDerivationDenial> {
        Ok(self.effect_name == pending_delivery.effect_name()
            && self
                .trigger_commit_identity
                .is_same_current_identity_as(pending_delivery.commit_identity())
            && self.pending_delivery_digest == hash_effect_pending_delivery(pending_delivery)?)
    }
}

impl WorthQueryAuthoritativeMutationExecutionBinding {
    pub(crate) fn from_handoff(
        handoff: WorthQueryAuthoritativeMutationExecutionHandoff,
    ) -> Result<Self, IdentityDerivationDenial> {
        let binding_digest = intent_execution_binding_identity(
            "authoritative-mutation-execution",
            handoff.handoff_digest(),
            None,
        )?;
        Ok(Self {
            handoff,
            binding_digest,
        })
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.handoff.family()
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.handoff.entrypoint()
    }

    pub fn execution_seam(&self) -> WorthQueryIntentAdmissionExecutionSeam {
        self.handoff.execution_seam()
    }

    pub fn command(&self) -> &WorthQueryWriteCommand {
        self.handoff.command()
    }

    pub fn handoff(&self) -> &WorthQueryAuthoritativeMutationExecutionHandoff {
        &self.handoff
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }
}

impl WorthQueryAuthoritativeMutationBatchExecutionBinding {
    pub(crate) fn from_handoff(
        handoff: WorthQueryAuthoritativeMutationBatchExecutionHandoff,
    ) -> Result<Self, IdentityDerivationDenial> {
        let binding_digest = intent_execution_binding_identity(
            "authoritative-mutation-batch-execution",
            handoff.handoff_digest(),
            None,
        )?;
        Ok(Self {
            handoff,
            binding_digest,
        })
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.handoff.family()
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.handoff.entrypoint()
    }

    pub fn execution_seam(&self) -> WorthQueryIntentAdmissionExecutionSeam {
        self.handoff.execution_seam()
    }

    pub fn handoff(&self) -> &WorthQueryAuthoritativeMutationBatchExecutionHandoff {
        &self.handoff
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }
}

fn hash_effect_pending_delivery(
    pending_delivery: &WorthQueryEffectDelivery,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceipt)
            .field_shape(WorthQueryEvidenceTag::new("role"), "pending-delivery")
            .field_value(
                WorthQueryEvidenceTag::new("effect"),
                pending_delivery.effect_name(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("commit"),
                &pending_delivery.commit_identity().evidence_identity(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("trigger_source"),
                pending_delivery.trigger_source(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("trigger_source_kind"),
                pending_delivery.trigger_source_kind().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("target"),
                pending_delivery.target(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("action"),
                pending_delivery.action().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("authority"),
                pending_delivery.authority_lane().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("policy"),
                pending_delivery.effect_policy().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("suppression"),
                pending_delivery.suppression_policy().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                effect_delivery_family_label(pending_delivery.family()),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("phase"),
                pending_delivery
                    .phase_evidence()
                    .phases()
                    .iter()
                    .map(|phase| phase.as_str()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("loop_prevention"),
                pending_delivery.phase_evidence().loop_prevention().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("source_lane"),
                WorthQueryIntentSourceLane::EffectTriggered.as_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("aspect"),
                terminal_aspect_touch_digest_parts(pending_delivery.aspect_touches())
                    .iter()
                    .map(String::as_str),
            )
            .field_value(
                WorthQueryEvidenceTag::new("payload"),
                pending_delivery.payload().terminal_digest_material(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("reason"),
                pending_delivery.reason(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("write_adjacent_trigger"),
                pending_delivery.write_adjacent_trigger().identity(),
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}

fn terminal_aspect_touch_digest_parts(touches: &[WorthQueryAspectTouch]) -> Vec<String> {
    touches
        .iter()
        .map(WorthQueryAspectTouch::admitted_touch_digest_part)
        .collect()
}

fn effect_delivery_family_label(
    family: &crate::runtime::WorthQueryEffectDeliveryFamily,
) -> &'static str {
    match family {
        crate::runtime::WorthQueryEffectDeliveryFamily::Delivered => "delivered",
        crate::runtime::WorthQueryEffectDeliveryFamily::PendingWriteIntent => {
            "pending_write_intent"
        }
        crate::runtime::WorthQueryEffectDeliveryFamily::Suppressed => "suppressed",
        crate::runtime::WorthQueryEffectDeliveryFamily::ExpressionFailed => "expression_failed",
    }
}

fn intent_execution_binding_identity(
    role: &'static str,
    handoff_digest: &str,
    pending_delivery_digest: Option<&str>,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceipt)
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_value(WorthQueryEvidenceTag::new("handoff"), handoff_digest)
            .optional_value(
                WorthQueryEvidenceTag::new("pending_delivery"),
                pending_delivery_digest,
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}

pub(super) fn handoff_execution_binding_identity(
    role: &'static str,
    handoff_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceipt)
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_value(WorthQueryEvidenceTag::new("handoff"), handoff_digest)
            .seal()?
            .as_str()
            .to_string(),
    )
}
