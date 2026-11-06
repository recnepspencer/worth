use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryBatchWriteReceipt, WorthQueryBranchIntentReceipt, WorthQueryEffectHandle,
    WorthQueryEffectIntentReceipt, WorthQueryInspectionTarget, WorthQueryIntentDenialEvidence,
    WorthQueryIntentReceipt, WorthQueryLiveView, WorthQueryPreviewHandleBindingEvidence,
    WorthQueryPreviewIntentReceipt, WorthQueryPreviewOutcome, WorthQueryWriteReceipt,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGenericInspectionIntentTargetSeed {
    LiveView { view_name: String },
    Effect { effect_name: String },
    WriteReceipt(WorthQueryWriteReceipt),
    BatchWriteReceipt(WorthQueryBatchWriteReceipt),
    IntentReceipt(WorthQueryIntentReceipt),
    IntentDenial(WorthQueryIntentDenialEvidence),
    EffectIntentReceipt(WorthQueryEffectIntentReceipt),
    PreviewBinding(WorthQueryPreviewHandleBindingEvidence),
    PreviewOutcome(WorthQueryPreviewOutcome),
    PreviewIntentReceipt(WorthQueryPreviewIntentReceipt),
    BranchIntentReceipt(WorthQueryBranchIntentReceipt),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGenericInspectionRequestLabel(String);

impl WorthQueryGenericInspectionRequestLabel {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for WorthQueryGenericInspectionRequestLabel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGenericInspectionIntentSeed {
    target: WorthQueryGenericInspectionIntentTargetSeed,
    request_label: WorthQueryGenericInspectionRequestLabel,
    request_input_digest: String,
}

pub trait WorthQueryGenericInspectionIntentTarget<'a> {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed;
}

impl WorthQueryGenericInspectionIntentSeed {
    pub(crate) fn from_target(target: WorthQueryInspectionTarget<'_>) -> Option<Self> {
        match target {
            WorthQueryInspectionTarget::LiveView { name } => Some(Self::from_live_view_name(name)),
            WorthQueryInspectionTarget::DerivedView { .. } => None,
            WorthQueryInspectionTarget::Effect { name } => Some(Self::from_effect_name(name)),
            WorthQueryInspectionTarget::WriteReceipt(receipt) => Some(Self::write_receipt(receipt)),
            WorthQueryInspectionTarget::BatchWriteReceipt(receipt) => {
                Some(Self::batch_write_receipt(receipt))
            }
            WorthQueryInspectionTarget::IntentReceipt(receipt) => {
                Some(Self::intent_receipt(receipt))
            }
            WorthQueryInspectionTarget::IntentDenial(evidence) => {
                Some(Self::intent_denial(evidence))
            }
            WorthQueryInspectionTarget::EffectIntentReceipt(receipt) => {
                Some(Self::effect_intent_receipt(receipt))
            }
            WorthQueryInspectionTarget::PreviewBinding(binding) => {
                Some(Self::preview_binding(binding))
            }
            WorthQueryInspectionTarget::PreviewOutcome(outcome) => {
                Some(Self::preview_outcome(outcome))
            }
            WorthQueryInspectionTarget::PreviewIntentReceipt(receipt) => {
                Some(Self::preview_intent_receipt(receipt))
            }
            WorthQueryInspectionTarget::BranchIntentReceipt(receipt) => {
                Some(Self::branch_intent_receipt(receipt))
            }
            WorthQueryInspectionTarget::AdmittedWorldBasis(_)
            | WorthQueryInspectionTarget::ScopedObservationBasis(_)
            | WorthQueryInspectionTarget::ScopedMutationPreparationBasis(_)
            | WorthQueryInspectionTarget::ScopedInspectionBasis(_)
            | WorthQueryInspectionTarget::ScopedReplayBasis(_)
            | WorthQueryInspectionTarget::ScopedMaterializationBasis(_)
            | WorthQueryInspectionTarget::ScopedSubscriptionDeclarationBasis(_)
            | WorthQueryInspectionTarget::ScopedSubscriptionActivationBasis(_)
            | WorthQueryInspectionTarget::ScopedPreviewCloseoutBasis(_) => None,
        }
    }

    pub(crate) fn live_view<T>(view: &WorthQueryLiveView<T>) -> Self {
        Self::from_live_view_name(view.name())
    }

    fn from_live_view_name(view_name: &str) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::LiveView {
                view_name: view_name.to_string(),
            },
            WorthQueryGenericInspectionRequestLabel::new(format!("inspect.live.{view_name}")),
            generic_inspection_seed_identity("live_view")
                .field_shape(WorthQueryEvidenceTag::new("view_name"), view_name)
                .seal()?
                .as_str()
                .to_string(),
        ))
    }

    pub(crate) fn effect<T>(effect: &WorthQueryEffectHandle<T>) -> Self {
        Self::from_effect_name(effect.name())
    }

    fn from_effect_name(effect_name: &str) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::Effect {
                effect_name: effect_name.to_string(),
            },
            WorthQueryGenericInspectionRequestLabel::new(format!("inspect.effect.{effect_name}")),
            generic_inspection_seed_identity("effect")
                .field_shape(WorthQueryEvidenceTag::new("effect_name"), effect_name)
                .seal()?
                .as_str()
                .to_string(),
        ))
    }

    pub(crate) fn write_receipt(
        receipt: &WorthQueryWriteReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::WriteReceipt(receipt.clone()),
            WorthQueryGenericInspectionRequestLabel::new(format!(
                "inspect.write_receipt.{}",
                receipt
                    .commit_identity()
                    .evidence_identity()
                    .reporting_projection()
            )),
            generic_inspection_seed_identity("write_receipt")
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("commit_identity"),
                    receipt.commit_evidence_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("snapshot_token"),
                    receipt.snapshot_evidence_identity(),
                )
                .seal()?
                .as_str()
                .to_string(),
        ))
    }

    pub(crate) fn batch_write_receipt(
        receipt: &WorthQueryBatchWriteReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::BatchWriteReceipt(receipt.clone()),
            WorthQueryGenericInspectionRequestLabel::new(format!(
                "inspect.batch_write_receipt.{}",
                receipt.batch_digest()
            )),
            generic_inspection_seed_identity("batch_write_receipt")
                .field_value(
                    WorthQueryEvidenceTag::new("batch_receipt"),
                    receipt.batch_digest(),
                )
                .seal()?
                .as_str()
                .to_string(),
        ))
    }

    pub(crate) fn intent_receipt(receipt: &WorthQueryIntentReceipt) -> Self {
        Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::IntentReceipt(receipt.clone()),
            WorthQueryGenericInspectionRequestLabel::new(format!(
                "inspect.intent_receipt.{}",
                receipt.intent_name()
            )),
            receipt_seed_input_digest("intent_receipt", receipt.receipt_identity()),
        )
    }

    pub(crate) fn intent_denial(evidence: &WorthQueryIntentDenialEvidence) -> Self {
        Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::IntentDenial(evidence.clone()),
            WorthQueryGenericInspectionRequestLabel::new(format!(
                "inspect.intent_denial.{}",
                evidence.intent_name()
            )),
            receipt_seed_input_digest("intent_denial", evidence.denial_digest()),
        )
    }

    pub(crate) fn effect_intent_receipt(receipt: &WorthQueryEffectIntentReceipt) -> Self {
        Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::EffectIntentReceipt(receipt.clone()),
            WorthQueryGenericInspectionRequestLabel::new(format!(
                "inspect.effect_intent_receipt.{}",
                receipt.effect_name()
            )),
            receipt_seed_input_digest("effect_intent_receipt", receipt.receipt_identity()),
        )
    }

    pub(crate) fn preview_binding(
        binding: &WorthQueryPreviewHandleBindingEvidence,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::PreviewBinding(binding.clone()),
            WorthQueryGenericInspectionRequestLabel::new(format!(
                "inspect.preview_binding.{}",
                binding.session_label().display()
            )),
            generic_inspection_seed_identity("preview_binding")
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("label_identity"),
                    binding.label_identity(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("handle_name"),
                    binding.handle_name(),
                )
                .seal()?
                .as_str()
                .to_string(),
        ))
    }

    pub(crate) fn preview_outcome(
        outcome: &WorthQueryPreviewOutcome,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::PreviewOutcome(outcome.clone()),
            WorthQueryGenericInspectionRequestLabel::new(format!(
                "inspect.preview_outcome.{}",
                outcome.label()
            )),
            generic_inspection_seed_identity("preview_outcome")
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("label_identity"),
                    outcome.session_label().identity_digest(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("write_count"),
                    outcome.write_count(),
                )
                .field_bool(WorthQueryEvidenceTag::new("promoted"), outcome.promoted())
                .field_bool(WorthQueryEvidenceTag::new("discarded"), outcome.discarded())
                .seal()?
                .as_str()
                .to_string(),
        ))
    }

    pub(crate) fn preview_intent_receipt(receipt: &WorthQueryPreviewIntentReceipt) -> Self {
        Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::PreviewIntentReceipt(receipt.clone()),
            WorthQueryGenericInspectionRequestLabel::new(format!(
                "inspect.preview_intent_receipt.{}",
                receipt.intent_name()
            )),
            receipt_seed_input_digest("preview_intent_receipt", receipt.receipt_identity()),
        )
    }

    pub(crate) fn branch_intent_receipt(receipt: &WorthQueryBranchIntentReceipt) -> Self {
        Self::new(
            WorthQueryGenericInspectionIntentTargetSeed::BranchIntentReceipt(receipt.clone()),
            WorthQueryGenericInspectionRequestLabel::new(format!(
                "inspect.branch_intent_receipt.{}",
                receipt.intent_name()
            )),
            receipt_seed_input_digest("branch_intent_receipt", receipt.receipt_identity()),
        )
    }

    fn new(
        target: WorthQueryGenericInspectionIntentTargetSeed,
        request_label: WorthQueryGenericInspectionRequestLabel,
        request_input_digest: String,
    ) -> Self {
        Self {
            target,
            request_label,
            request_input_digest,
        }
    }

    pub fn target(&self) -> &WorthQueryGenericInspectionIntentTargetSeed {
        &self.target
    }

    pub fn request_label(&self) -> &WorthQueryGenericInspectionRequestLabel {
        &self.request_label
    }

    pub fn request_input_digest(&self) -> &str {
        &self.request_input_digest
    }
}

fn receipt_seed_input_digest(
    target_family: &'static str,
    receipt_identity: &WorthQueryEvidenceIdentity,
) -> Result<String, IdentityDerivationDenial> {
    Ok(generic_inspection_seed_identity(target_family)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("receipt_digest"),
            receipt_identity,
        )
        .seal()?
        .as_str()
        .to_string())
}

fn generic_inspection_seed_identity(
    target_family: &'static str,
) -> crate::evidence_identity::WorthQueryEvidenceIdentityEncoder {
    worth_query_evidence_identity(WorthQueryEvidenceScope::GenericInspectionIntentSeed)
        .field_shape(WorthQueryEvidenceTag::new("target_family"), target_family)
}

impl<'a, T> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryLiveView<T> {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::live_view(self)
    }
}

impl<'a, T> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryEffectHandle<T> {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::effect(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryWriteReceipt {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::write_receipt(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryBatchWriteReceipt {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::batch_write_receipt(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryIntentReceipt {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::intent_receipt(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryIntentDenialEvidence {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::intent_denial(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryEffectIntentReceipt {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::effect_intent_receipt(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a>
    for &'a WorthQueryPreviewHandleBindingEvidence
{
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::preview_binding(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryPreviewOutcome {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::preview_outcome(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryPreviewIntentReceipt {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::preview_intent_receipt(self)
    }
}

impl<'a> WorthQueryGenericInspectionIntentTarget<'a> for &'a WorthQueryBranchIntentReceipt {
    fn into_generic_inspection_intent_seed(self) -> WorthQueryGenericInspectionIntentSeed {
        WorthQueryGenericInspectionIntentSeed::branch_intent_receipt(self)
    }
}
