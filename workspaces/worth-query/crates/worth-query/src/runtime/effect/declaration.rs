use super::super::{
    WorthQueryAuthorityLane, WorthQueryDerivedViewHandle, WorthQueryEffectAction,
    WorthQueryEffectPolicy, WorthQueryLiveView,
};
use super::follow_on::{
    WorthQueryEffectWriteAdjacentTrigger, WorthQueryEffectWriteAdjacentTriggerClass,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::runtime::WorthQueryAspectTouch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEffectTrigger {
    pub(in crate::runtime::effect) source: WorthQueryEffectTriggerSource,
    aspects: Vec<WorthQueryAspectTouch>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::runtime::effect) enum WorthQueryEffectTriggerSource {
    LiveView { view_name: String },
    ComputedView { view_name: String },
}
impl WorthQueryEffectTrigger {
    pub fn live_view<T>(
        view: &WorthQueryLiveView<T>,
        aspects: impl IntoIterator<Item = WorthQueryAspectTouch>,
    ) -> Self {
        Self {
            source: WorthQueryEffectTriggerSource::LiveView {
                view_name: view.name().to_string(),
            },
            aspects: aspects.into_iter().collect(),
        }
    }
    pub fn computed_view<T>(
        view: &WorthQueryDerivedViewHandle<T>,
        aspects: impl IntoIterator<Item = WorthQueryAspectTouch>,
    ) -> Self {
        Self {
            source: WorthQueryEffectTriggerSource::ComputedView {
                view_name: view.name().to_string(),
            },
            aspects: aspects.into_iter().collect(),
        }
    }
    #[cfg(test)]
    pub(in crate::runtime) fn live_view_name(
        view_name: impl Into<String>,
        aspects: impl IntoIterator<Item = WorthQueryAspectTouch>,
    ) -> Self {
        Self {
            source: WorthQueryEffectTriggerSource::LiveView {
                view_name: view_name.into(),
            },
            aspects: aspects.into_iter().collect(),
        }
    }
    pub fn source_name(&self) -> &str {
        match &self.source {
            WorthQueryEffectTriggerSource::LiveView { view_name }
            | WorthQueryEffectTriggerSource::ComputedView { view_name } => view_name,
        }
    }
    pub fn aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.aspects
    }
    pub fn source_kind(&self) -> WorthQueryEffectTriggerSourceKind {
        match self.source {
            WorthQueryEffectTriggerSource::LiveView { .. } => {
                WorthQueryEffectTriggerSourceKind::LiveView
            }
            WorthQueryEffectTriggerSource::ComputedView { .. } => {
                WorthQueryEffectTriggerSourceKind::ComputedView
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEffectTriggerSourceKind {
    LiveView,
    ComputedView,
}
impl WorthQueryEffectTriggerSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LiveView => "live-view",
            Self::ComputedView => "computed-view",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryEffectCondition {
    Always,
    Expression(WorthQueryEffectExpression),
}
impl WorthQueryEffectCondition {
    pub fn always() -> Self {
        Self::Always
    }
    pub fn expression(
        descriptor: impl Into<String>,
        input_aspects: impl IntoIterator<Item = WorthQueryAspectTouch>,
        output_aspects: impl IntoIterator<Item = WorthQueryAspectTouch>,
    ) -> Self {
        Self::Expression(WorthQueryEffectExpression {
            descriptor: descriptor.into(),
            input_aspects: input_aspects.into_iter().collect(),
            output_aspects: output_aspects.into_iter().collect(),
            failure_posture: WorthQueryEffectExpressionFailurePosture::Admitted,
        })
    }
    pub fn failing_expression(
        descriptor: impl Into<String>,
        input_aspects: impl IntoIterator<Item = WorthQueryAspectTouch>,
        output_aspects: impl IntoIterator<Item = WorthQueryAspectTouch>,
    ) -> Self {
        Self::Expression(WorthQueryEffectExpression {
            descriptor: descriptor.into(),
            input_aspects: input_aspects.into_iter().collect(),
            output_aspects: output_aspects.into_iter().collect(),
            failure_posture: WorthQueryEffectExpressionFailurePosture::DeterministicFailure,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEffectExpression {
    descriptor: String,
    input_aspects: Vec<WorthQueryAspectTouch>,
    output_aspects: Vec<WorthQueryAspectTouch>,
    failure_posture: WorthQueryEffectExpressionFailurePosture,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEffectExpressionFailurePosture {
    Admitted,
    DeterministicFailure,
}
impl WorthQueryEffectExpression {
    pub fn descriptor(&self) -> &str {
        &self.descriptor
    }
    pub fn input_aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.input_aspects
    }
    pub fn output_aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.output_aspects
    }
    pub fn failure_posture(&self) -> WorthQueryEffectExpressionFailurePosture {
        self.failure_posture
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEffectSuppressionPolicy {
    None,
    MeaningfulSemanticDelta,
}
impl WorthQueryEffectSuppressionPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::MeaningfulSemanticDelta => "meaningful-semantic-delta",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEffectDeclaration {
    name: String,
    trigger: WorthQueryEffectTrigger,
    condition: WorthQueryEffectCondition,
    action: WorthQueryEffectAction,
    target_lane: WorthQueryAuthorityLane,
    target: String,
    effect_policy: WorthQueryEffectPolicy,
    suppression_policy: WorthQueryEffectSuppressionPolicy,
    write_adjacent_trigger: WorthQueryEffectWriteAdjacentTrigger,
}
impl WorthQueryEffectDeclaration {
    pub fn deliver(
        name: impl Into<String>,
        trigger: WorthQueryEffectTrigger,
        target: impl Into<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let name = name.into();
        Ok(Self {
            write_adjacent_trigger: WorthQueryEffectWriteAdjacentTrigger::ordinary(&name)?,
            name,
            trigger,
            condition: WorthQueryEffectCondition::Always,
            action: WorthQueryEffectAction::Deliver,
            target_lane: WorthQueryAuthorityLane::EffectDeliveryState,
            target: target.into(),
            effect_policy: WorthQueryEffectPolicy::AuthoritativeAllowed,
            suppression_policy: WorthQueryEffectSuppressionPolicy::None,
        })
    }
    pub fn write_intent(
        name: impl Into<String>,
        trigger: WorthQueryEffectTrigger,
        intent: impl Into<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let name = name.into();
        Ok(Self {
            write_adjacent_trigger: WorthQueryEffectWriteAdjacentTrigger::ordinary(&name)?,
            name,
            trigger,
            condition: WorthQueryEffectCondition::Always,
            action: WorthQueryEffectAction::WriteIntent,
            target_lane: WorthQueryAuthorityLane::PendingWriteIntent,
            target: intent.into(),
            effect_policy: WorthQueryEffectPolicy::SandboxedWriteIntent,
            suppression_policy: WorthQueryEffectSuppressionPolicy::None,
        })
    }
    pub fn with_condition(mut self, condition: WorthQueryEffectCondition) -> Self {
        self.condition = condition;
        self
    }
    #[cfg(test)]
    pub(in crate::runtime) fn with_effect_policy(
        mut self,
        effect_policy: WorthQueryEffectPolicy,
    ) -> Self {
        self.effect_policy = effect_policy;
        self
    }
    #[cfg(test)]
    pub(in crate::runtime) fn with_target_lane(
        mut self,
        target_lane: WorthQueryAuthorityLane,
    ) -> Self {
        self.target_lane = target_lane;
        self
    }
    pub fn with_meaningful_change_suppression(mut self) -> Self {
        self.suppression_policy = WorthQueryEffectSuppressionPolicy::MeaningfulSemanticDelta;
        self
    }
    pub fn with_write_adjacent_trigger(
        mut self,
        class: WorthQueryEffectWriteAdjacentTriggerClass,
        origin_identity: WorthQueryEvidenceIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        self.write_adjacent_trigger =
            WorthQueryEffectWriteAdjacentTrigger::new(class, origin_identity)?;
        Ok(self)
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn trigger(&self) -> &WorthQueryEffectTrigger {
        &self.trigger
    }
    pub fn condition(&self) -> &WorthQueryEffectCondition {
        &self.condition
    }
    pub fn action(&self) -> WorthQueryEffectAction {
        self.action
    }
    pub fn target_lane(&self) -> WorthQueryAuthorityLane {
        self.target_lane
    }
    pub fn target(&self) -> &str {
        &self.target
    }
    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }
    pub fn suppression_policy(&self) -> WorthQueryEffectSuppressionPolicy {
        self.suppression_policy
    }
    pub fn write_adjacent_trigger(&self) -> &WorthQueryEffectWriteAdjacentTrigger {
        &self.write_adjacent_trigger
    }
}
