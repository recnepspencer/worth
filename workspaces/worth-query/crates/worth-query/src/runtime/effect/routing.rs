#[path = "routing_aspects.rs"]
mod routing_aspects;

use std::collections::{BTreeMap, BTreeSet};

use crate::memory_workspace::WorthQueryMutationReceipt;

use super::super::{
    WorthQueryAuthorityLane, WorthQueryDerivedViewRuntime, WorthQueryEffectAction,
    WorthQueryRuntimeError,
};
use super::declaration::{
    WorthQueryEffectCondition, WorthQueryEffectDeclaration,
    WorthQueryEffectExpressionFailurePosture, WorthQueryEffectSuppressionPolicy,
    WorthQueryEffectTriggerSource, WorthQueryEffectTriggerSourceKind,
};
use super::delivery::{WorthQueryEffectDelivery, WorthQueryEffectPayload};
use super::registry::{WorthQueryEffectIndex, WorthQueryEffectRuntime, WorthQueryEffectTarget};
use crate::runtime::{WorthQueryAspectTouch, WorthQueryMutationTargetCollectionIdentity};
use crate::runtime::{WorthQueryDerivedMaterializationTarget, WorthQueryLiveArtifactTarget};
use routing_aspects::{aspects_match, insert_declared_aspects};

pub(in crate::runtime) fn admit_effect_declaration(
    live_view_targets: &BTreeSet<WorthQueryLiveArtifactTarget>,
    computed_view_targets: &BTreeSet<WorthQueryDerivedMaterializationTarget>,
    declaration: &WorthQueryEffectDeclaration,
) -> Result<(), WorthQueryRuntimeError> {
    if declaration.name().is_empty() {
        return Err(effect_declaration_error(
            declaration.name(),
            "name-admission",
            "effect name may not be empty",
        ));
    }
    declaration
        .effect_policy()
        .admit(declaration.action(), declaration.target_lane())
        .map_err(WorthQueryRuntimeError::EffectPolicyDenied)?;
    match (declaration.action(), declaration.target_lane()) {
        (WorthQueryEffectAction::Deliver, WorthQueryAuthorityLane::EffectDeliveryState) => {}
        (WorthQueryEffectAction::WriteIntent, WorthQueryAuthorityLane::PendingWriteIntent) => {}
        (WorthQueryEffectAction::WriteIntent, _) => {
            return Err(effect_declaration_error(
                declaration.name(),
                "write-intent-admission",
                "effect-triggered writes must lower to pending write intent authority before any commit",
            ));
        }
        _ => {
            return Err(effect_declaration_error(
                declaration.name(),
                "authority-admission",
                "effect declarations may only deliver into effect delivery state or lower into pending write intent authority",
            ));
        }
    }
    if declaration.trigger().aspect_touches().is_empty() {
        return Err(effect_declaration_error(
            declaration.name(),
            "trigger-admission",
            "effect trigger must declare at least one aspect",
        ));
    }
    match &declaration.trigger().source {
        WorthQueryEffectTriggerSource::LiveView { view_name } => {
            let target = WorthQueryLiveArtifactTarget::from_view_name(view_name.clone());
            if !live_view_targets.contains(&target) {
                return Err(effect_declaration_error(
                    declaration.name(),
                    "trigger-admission",
                    format!("live trigger `{view_name}` is not declared"),
                ));
            }
        }
        WorthQueryEffectTriggerSource::ComputedView { view_name } => {
            let target = WorthQueryDerivedMaterializationTarget::new(view_name.clone());
            if !computed_view_targets.contains(&target) {
                return Err(effect_declaration_error(
                    declaration.name(),
                    "trigger-admission",
                    format!("computed trigger `{view_name}` is not declared"),
                ));
            }
        }
    }
    if let WorthQueryEffectCondition::Expression(expression) = declaration.condition() {
        if expression.descriptor().is_empty() {
            return Err(effect_declaration_error(
                declaration.name(),
                "condition-admission",
                "expression descriptor may not be empty",
            ));
        }
        if expression.input_aspect_touches().is_empty()
            || expression.output_aspect_touches().is_empty()
        {
            return Err(effect_declaration_error(
                declaration.name(),
                "condition-admission",
                "expression conditions must declare input and output aspects",
            ));
        }
    }
    Ok(())
}
fn effect_declaration_error(
    effect_name: &str,
    stage: &'static str,
    message: impl Into<String>,
) -> WorthQueryRuntimeError {
    WorthQueryRuntimeError::EffectDeclaration {
        effect_name: effect_name.to_string(),
        stage,
        message: message.into(),
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::runtime) struct WorthQueryEffectRouteResult {
    considered_effect_count: usize,
    delivered_effect_count: usize,
    pending_write_intent_count: usize,
    suppressed_effect_count: usize,
    meaningful_suppression_count: usize,
    expression_failure_count: usize,
}
impl WorthQueryEffectRouteResult {
    pub(in crate::runtime) fn considered_effect_count(&self) -> usize {
        self.considered_effect_count
    }
    pub(in crate::runtime) fn delivered_effect_count(&self) -> usize {
        self.delivered_effect_count
    }
    pub(in crate::runtime) fn pending_write_intent_count(&self) -> usize {
        self.pending_write_intent_count
    }
    pub(in crate::runtime) fn suppressed_effect_count(&self) -> usize {
        self.suppressed_effect_count
    }
    pub(in crate::runtime) fn meaningful_suppression_count(&self) -> usize {
        self.meaningful_suppression_count
    }
    pub(in crate::runtime) fn expression_failure_count(&self) -> usize {
        self.expression_failure_count
    }
}
pub(in crate::runtime) fn route_effect_deliveries(
    effects: &mut BTreeMap<WorthQueryEffectTarget, WorthQueryEffectRuntime>,
    effect_index: &WorthQueryEffectIndex,
    derived_views: &BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>,
    live_artifact_target_collections: &BTreeMap<
        WorthQueryLiveArtifactTarget,
        WorthQueryMutationTargetCollectionIdentity,
    >,
    receipt: &WorthQueryMutationReceipt,
    affected_live_view_targets: &[WorthQueryLiveArtifactTarget],
    affected_derived_view_targets: &[WorthQueryDerivedMaterializationTarget],
) -> WorthQueryEffectRouteResult {
    let mut candidates: BTreeSet<WorthQueryEffectTarget> = BTreeSet::new();
    candidates.extend(effect_index.live_candidates(affected_live_view_targets.iter()));
    candidates.extend(effect_index.computed_candidates(affected_derived_view_targets.iter()));
    let mut result = WorthQueryEffectRouteResult::default();
    for effect_name in candidates {
        let Some(effect) = effects.get_mut(&effect_name) else {
            continue;
        };
        result.considered_effect_count += 1;
        effect.counters.considered += 1;
        let trigger = collect_trigger_change(
            &effect.declaration,
            derived_views,
            live_artifact_target_collections,
            receipt,
        );
        let Some(trigger) = trigger else {
            let reason = match effect.declaration.suppression_policy() {
                WorthQueryEffectSuppressionPolicy::MeaningfulSemanticDelta => {
                    "meaningful semantic delta suppression: trigger source changed but declared aspects did not"
                }
                WorthQueryEffectSuppressionPolicy::None => {
                    "trigger source changed but declared aspects did not"
                }
            };
            let suppressed = WorthQueryEffectDelivery::suppressed(
                &effect.declaration,
                receipt.commit_identity.clone(),
                effect.declaration.trigger().source_name(),
                effect.declaration.trigger().source_kind(),
                reason,
            );
            effect.record_delivery(suppressed);
            effect.counters.suppressed += 1;
            result.suppressed_effect_count += 1;
            if effect.declaration.suppression_policy()
                == WorthQueryEffectSuppressionPolicy::MeaningfulSemanticDelta
            {
                effect.counters.meaningful_suppressions += 1;
                result.meaningful_suppression_count += 1;
            }
            continue;
        };
        match evaluate_condition(effect.declaration.condition(), &trigger.aspect_touches) {
            EffectConditionOutcome::Delivered(payload) => match effect.declaration.action() {
                WorthQueryEffectAction::Deliver => {
                    let delivery = WorthQueryEffectDelivery::delivered(
                        &effect.declaration,
                        receipt.commit_identity.clone(),
                        trigger.source,
                        trigger.source_kind,
                        trigger.aspect_touches,
                        payload,
                    );
                    effect.record_delivery(delivery);
                    effect.counters.delivered += 1;
                    result.delivered_effect_count += 1;
                }
                WorthQueryEffectAction::WriteIntent => {
                    let delivery = WorthQueryEffectDelivery::pending_write_intent(
                        &effect.declaration,
                        receipt.commit_identity.clone(),
                        trigger.source,
                        trigger.source_kind,
                        trigger.aspect_touches,
                        payload,
                    );
                    effect.record_delivery(delivery);
                    effect.counters.pending_write_intents += 1;
                    result.pending_write_intent_count += 1;
                }
                WorthQueryEffectAction::Derive => {
                    let delivery = WorthQueryEffectDelivery::suppressed(
                            &effect.declaration,
                            receipt.commit_identity.clone(),
                            trigger.source,
                            trigger.source_kind,
                            "derive-only effects are admitted by policy but not executable as runtime deliveries",
                        );
                    effect.record_delivery(delivery);
                    effect.counters.suppressed += 1;
                    result.suppressed_effect_count += 1;
                }
            },
            EffectConditionOutcome::Suppressed(reason) => {
                let delivery = WorthQueryEffectDelivery::suppressed(
                    &effect.declaration,
                    receipt.commit_identity.clone(),
                    trigger.source,
                    trigger.source_kind,
                    reason,
                );
                effect.record_delivery(delivery);
                effect.counters.suppressed += 1;
                result.suppressed_effect_count += 1;
            }
            EffectConditionOutcome::Failed(reason) => {
                let delivery = WorthQueryEffectDelivery::expression_failed(
                    &effect.declaration,
                    receipt.commit_identity.clone(),
                    trigger.source,
                    trigger.source_kind,
                    trigger.aspect_touches,
                    reason,
                );
                effect.record_delivery(delivery);
                effect.counters.expression_failures += 1;
                result.expression_failure_count += 1;
            }
        }
    }
    result
}
struct TriggerChange {
    source: String,
    source_kind: WorthQueryEffectTriggerSourceKind,
    aspect_touches: Vec<WorthQueryAspectTouch>,
}
fn collect_trigger_change(
    declaration: &WorthQueryEffectDeclaration,
    derived_views: &BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>,
    live_artifact_target_collections: &BTreeMap<
        WorthQueryLiveArtifactTarget,
        WorthQueryMutationTargetCollectionIdentity,
    >,
    receipt: &WorthQueryMutationReceipt,
) -> Option<TriggerChange> {
    match &declaration.trigger().source {
        WorthQueryEffectTriggerSource::LiveView { view_name } => {
            let aspects = declaration.trigger().aspect_touches();
            let live_target = WorthQueryLiveArtifactTarget::from_view_name(view_name.clone());
            let collection = live_artifact_target_collections.get(&live_target)?;
            let mut changed_aspects = BTreeSet::new();
            for delta in &receipt.deltas {
                if !delta
                    .target_collection_identity()
                    .same_target_collection_as(collection)
                {
                    continue;
                }
                if delta.admitted_touched_aspects().is_empty() {
                    insert_declared_aspects(&mut changed_aspects, aspects);
                } else {
                    for touch in delta
                        .admitted_touched_aspects()
                        .iter()
                        .filter(|touch| aspects_match(aspects, touch))
                    {
                        changed_aspects.insert(touch.clone());
                    }
                }
            }
            (!changed_aspects.is_empty()).then(|| TriggerChange {
                source: view_name.clone(),
                source_kind: WorthQueryEffectTriggerSourceKind::LiveView,
                aspect_touches: changed_aspects.into_iter().collect(),
            })
        }
        WorthQueryEffectTriggerSource::ComputedView { view_name } => {
            let aspects = declaration.trigger().aspect_touches();
            let view =
                derived_views.get(&WorthQueryDerivedMaterializationTarget::new(view_name))?;
            let mut changed_aspects = BTreeSet::new();
            for patch in &view.patches {
                if !patch
                    .commit_identity()
                    .is_same_current_identity_as(&receipt.commit_identity)
                {
                    continue;
                }
                if patch.aspect_touches().is_empty() {
                    insert_declared_aspects(&mut changed_aspects, aspects);
                } else {
                    for aspect in patch
                        .aspect_touches()
                        .iter()
                        .filter(|touch| aspects_match(aspects, touch))
                    {
                        changed_aspects.insert(aspect.clone());
                    }
                }
            }
            (!changed_aspects.is_empty()).then(|| TriggerChange {
                source: view_name.clone(),
                source_kind: WorthQueryEffectTriggerSourceKind::ComputedView,
                aspect_touches: changed_aspects.into_iter().collect(),
            })
        }
    }
}
enum EffectConditionOutcome {
    Delivered(WorthQueryEffectPayload),
    Suppressed(String),
    Failed(String),
}
fn evaluate_condition(
    condition: &WorthQueryEffectCondition,
    changed_aspects: &[WorthQueryAspectTouch],
) -> EffectConditionOutcome {
    match condition {
        WorthQueryEffectCondition::Always => {
            EffectConditionOutcome::Delivered(WorthQueryEffectPayload::always(changed_aspects))
        }
        WorthQueryEffectCondition::Expression(expression) => {
            if expression.failure_posture()
                == WorthQueryEffectExpressionFailurePosture::DeterministicFailure
            {
                return EffectConditionOutcome::Failed(format!(
                    "expression `{}` reported deterministic failure",
                    expression.descriptor()
                ));
            }
            let input_hits = changed_aspects
                .iter()
                .any(|aspect| aspects_match(expression.input_aspect_touches(), aspect));
            if !input_hits {
                return EffectConditionOutcome::Suppressed(format!(
                    "expression `{}` inputs were not changed",
                    expression.descriptor()
                ));
            }
            EffectConditionOutcome::Delivered(WorthQueryEffectPayload::expression(
                expression.descriptor(),
                expression.input_aspect_touches(),
                expression.output_aspect_touches(),
                changed_aspects,
            ))
        }
    }
}
