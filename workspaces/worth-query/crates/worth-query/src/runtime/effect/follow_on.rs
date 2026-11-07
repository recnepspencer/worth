use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEffectWriteAdjacentTriggerClass {
    OrdinaryEffect,
    TimeOnlyWake,
    AsyncCompletion,
    MixedCause,
    ReplayDrift,
    RemaskDrift,
    PreviewCrossedResidue,
    StaleCompletion,
}

impl WorthQueryEffectWriteAdjacentTriggerClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OrdinaryEffect => "ordinary-effect",
            Self::TimeOnlyWake => "time-only-wake",
            Self::AsyncCompletion => "async-completion",
            Self::MixedCause => "mixed-cause",
            Self::ReplayDrift => "replay-drift",
            Self::RemaskDrift => "remask-drift",
            Self::PreviewCrossedResidue => "preview-crossed-residue",
            Self::StaleCompletion => "stale-completion",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEffectWriteAdjacentTrigger {
    class: WorthQueryEffectWriteAdjacentTriggerClass,
    origin_identity: WorthQueryEvidenceIdentity,
    identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryEffectWriteAdjacentTrigger {
    pub fn ordinary(effect_name: &str) -> Result<Self, IdentityDerivationDenial> {
        let origin_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceiptPhase)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "ordinary_effect_origin_v1",
                )
                .field_value(WorthQueryEvidenceTag::new("effect"), effect_name)
                .seal()?;
        Ok(Self::new(
            WorthQueryEffectWriteAdjacentTriggerClass::OrdinaryEffect,
            origin_identity,
        ))
    }

    pub fn new(
        class: WorthQueryEffectWriteAdjacentTriggerClass,
        origin_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        let identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceiptPhase)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "worth_query_effect_write_adjacent_trigger_v1",
                )
                .field_shape(WorthQueryEvidenceTag::new("class"), class.as_str())
                .field_evidence_identity(WorthQueryEvidenceTag::new("origin"), &origin_identity)
                .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens");
        Self {
            class,
            origin_identity,
            identity,
        }
    }

    pub fn class(&self) -> WorthQueryEffectWriteAdjacentTriggerClass {
        self.class
    }

    pub fn origin_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.origin_identity
    }

    pub fn digest(&self) -> &str {
        self.identity.as_str()
    }

    pub fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }
}
