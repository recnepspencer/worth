//! Bound rule, path, and request shapes in an installed capability plan.

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityCardinalityDimension, ApplicationCapabilityFieldDimension,
    ApplicationCapabilityValidityTimeline,
};
use worth_relational::facade::authorization::{
    RelationalAuthorizationPathPlan, RelationalAuthorizationTraversal,
};
use worth_relational::facade::identity::KindId;
use worth_runtime_bridge::facade::BridgeAuthorizationRuleContract;

#[derive(Clone)]
pub(in crate::domain_computation::authorization) struct WorthQueryCapabilityRuleBinding {
    bridge: BridgeAuthorizationRuleContract,
    path_requirements: Vec<Vec<usize>>,
    recorded_actor: Option<(worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRequirement, [u8;32])>,
}

impl WorthQueryCapabilityRuleBinding {
    pub(in crate::domain_computation::authorization) fn new(
        bridge: BridgeAuthorizationRuleContract,
        path_requirements: Vec<Vec<usize>>,
    ) -> Self {
        Self {
            bridge,
            path_requirements,
            recorded_actor: None,
        }
    }

    pub(in crate::domain_computation::authorization) fn recorded_actor(
        &self,
    ) -> Option<(worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRequirement, [u8;32])>{
        self.recorded_actor
    }
    pub(in crate::domain_computation::authorization) fn required_recorded_actor(
        requirement: worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRequirement,
        identity: [u8; 32],
    ) -> Self {
        use worth_runtime_bridge::facade::{
            BridgeAuthorizationClauseContract, BridgeAuthorizationRequirementContract,
            BridgeAuthorizationRuleEffect,
        };
        Self {
            bridge: BridgeAuthorizationRuleContract::all(
                BridgeAuthorizationRuleEffect::Required,
                [BridgeAuthorizationRequirementContract::any([
                    BridgeAuthorizationClauseContract::new(identity),
                ])],
            ),
            path_requirements: Vec::new(),
            recorded_actor: Some((requirement, identity)),
        }
    }
    pub(in crate::domain_computation::authorization) const fn bridge(
        &self,
    ) -> &BridgeAuthorizationRuleContract {
        &self.bridge
    }

    pub(in crate::domain_computation::authorization) fn path_requirements(&self) -> &[Vec<usize>] {
        &self.path_requirements
    }
}

#[derive(Clone)]
pub(in crate::domain_computation::authorization) struct WorthQueryCapabilityDecisionRuleBindings {
    pub(in crate::domain_computation::authorization) grant: usize,
    pub(in crate::domain_computation::authorization) allow: usize,
    pub(in crate::domain_computation::authorization) deny: Option<usize>,
    pub(in crate::domain_computation::authorization) conflict: Option<usize>,
    pub(in crate::domain_computation::authorization) separation_of_duty: Option<usize>,
    pub(in crate::domain_computation::authorization) distinct_actor: Option<usize>,
    pub(in crate::domain_computation::authorization) recorded_actor: Option<usize>,
}

#[derive(Clone, Copy)]
pub(in crate::domain_computation::authorization) struct WorthQueryCapabilityGrantWitnessBinding {
    path_index: usize,
    entity_ordinal: usize,
}

impl WorthQueryCapabilityGrantWitnessBinding {
    pub(in crate::domain_computation::authorization) const fn new(
        path_index: usize,
        entity_ordinal: usize,
    ) -> Self {
        Self {
            path_index,
            entity_ordinal,
        }
    }

    pub(in crate::domain_computation::authorization) const fn path_index(self) -> usize {
        self.path_index
    }

    pub(in crate::domain_computation::authorization) const fn entity_ordinal(self) -> usize {
        self.entity_ordinal
    }
}

pub(in crate::domain_computation::authorization) struct WorthQueryCapabilityPathTemplate {
    pub(in crate::domain_computation::authorization) plan: RelationalAuthorizationPathPlan,
    pub(in crate::domain_computation::authorization) identity: [u8; 32],
    pub(in crate::domain_computation::authorization) guard: WorthQueryCapabilityRequestGuard,
    pub(in crate::domain_computation::authorization) grant_ordinal: Option<usize>,
    pub(in crate::domain_computation::authorization) elevation_ordinals: Vec<usize>,
    pub(in crate::domain_computation::authorization) elevation_resource_ordinal: Option<usize>,
    pub(in crate::domain_computation::authorization) context_anchors:
        Vec<WorthQueryCapabilityContextAnchor>,
}

pub(in crate::domain_computation::authorization) enum WorthQueryCapabilityRequestGuard {
    Unconditional,
    Accepted {
        axis: WorthQueryCapabilityRequestValueAxis,
        values: Vec<AspectValue>,
    },
}

#[derive(Clone, Copy)]
pub(in crate::domain_computation::authorization) enum WorthQueryCapabilityRequestValueAxis {
    Action,
    Purpose,
    Field,
    Magnitude,
}

pub(in crate::domain_computation::authorization) struct WorthQueryCapabilityContextAnchor {
    pub(in crate::domain_computation::authorization) ordinal: usize,
    pub(in crate::domain_computation::authorization) kind: KindId,
    pub(in crate::domain_computation::authorization) context: String,
    pub(in crate::domain_computation::authorization) context_type: String,
    pub(in crate::domain_computation::authorization) slot: String,
    pub(in crate::domain_computation::authorization) slot_type: String,
    pub(in crate::domain_computation::authorization) entity: String,
}

pub(in crate::domain_computation::authorization) struct WorthQueryCapabilityRequestBindings {
    pub(in crate::domain_computation::authorization) action: AspectValue,
    pub(in crate::domain_computation::authorization) purpose: AspectValue,
    pub(in crate::domain_computation::authorization) resource_entity: String,
    pub(in crate::domain_computation::authorization) related_relation:
        Option<RelationalAuthorizationTraversal>,
    pub(in crate::domain_computation::authorization) field: Option<AspectFieldLocator>,
    pub(in crate::domain_computation::authorization) magnitude: Option<AspectFieldLocator>,
    pub(in crate::domain_computation::authorization) cardinality:
        ApplicationCapabilityCardinalityDimension,
    pub(in crate::domain_computation::authorization) timeline:
        ApplicationCapabilityValidityTimeline,
    pub(in crate::domain_computation::authorization) not_before: AspectFieldLocator,
    pub(in crate::domain_computation::authorization) not_after: AspectFieldLocator,
    pub(in crate::domain_computation::authorization) context: String,
    pub(in crate::domain_computation::authorization) context_type: String,
}

pub(in crate::domain_computation::authorization) const fn field_binding(
    dimension: &ApplicationCapabilityFieldDimension,
) -> Option<
    &worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldBinding,
> {
    match dimension {
        ApplicationCapabilityFieldDimension::NotApplicable => None,
        ApplicationCapabilityFieldDimension::Bound(binding) => Some(binding),
    }
}
