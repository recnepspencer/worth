//! Bound rule, path, and request shapes in an installed capability plan.

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityCardinalityDimension, ApplicationCapabilityFieldDimension,
    ApplicationCapabilityValidityTimeline, ApplicationRecordedActorComparison,
};
use worth_relational::facade::authorization::{
    RelationalAuthorizationPathPlan, RelationalAuthorizationTraversal,
};
use worth_relational::facade::identity::KindId;
use worth_runtime_bridge::facade::{
    BridgeAuthorizationRuleContract, BridgeRecordedActorComparison,
};

#[derive(Clone)]
pub struct WorthQueryCapabilityRuleBinding {
    bridge: BridgeAuthorizationRuleContract,
    path_requirements: Vec<Vec<usize>>,
    recorded_actor: Option<(worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRequirement, [u8;32])>,
}

impl WorthQueryCapabilityRuleBinding {
    pub fn new(
        bridge: BridgeAuthorizationRuleContract,
        path_requirements: Vec<Vec<usize>>,
    ) -> Self {
        Self {
            bridge,
            path_requirements,
            recorded_actor: None,
        }
    }

    pub fn recorded_actor(
        &self,
    ) -> Option<(worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRequirement, [u8;32])>{
        self.recorded_actor
    }
    pub fn required_recorded_actor(
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
                    BridgeAuthorizationClauseContract::recorded_actor(
                        identity,
                        bridge_recorded_actor_comparison(requirement.comparison()),
                    ),
                ])],
            ),
            path_requirements: Vec::new(),
            recorded_actor: Some((requirement, identity)),
        }
    }
    pub const fn bridge(&self) -> &BridgeAuthorizationRuleContract {
        &self.bridge
    }

    pub fn path_requirements(&self) -> &[Vec<usize>] {
        &self.path_requirements
    }
}

pub const fn bridge_recorded_actor_comparison(
    comparison: ApplicationRecordedActorComparison,
) -> BridgeRecordedActorComparison {
    match comparison {
        ApplicationRecordedActorComparison::SamePrincipal => {
            BridgeRecordedActorComparison::SamePrincipal
        }
        ApplicationRecordedActorComparison::DifferentPrincipal => {
            BridgeRecordedActorComparison::DifferentPrincipal
        }
    }
}

#[derive(Clone)]
pub struct WorthQueryCapabilityDecisionRuleBindings {
    pub grant: usize,
    pub allow: usize,
    pub deny: Option<usize>,
    pub conflict: Option<usize>,
    pub separation_of_duty: Option<usize>,
    pub distinct_actor: Option<usize>,
    pub recorded_actor: Option<usize>,
}

#[derive(Clone, Copy)]
pub struct WorthQueryCapabilityGrantWitnessBinding {
    path_index: usize,
    entity_ordinal: usize,
}

impl WorthQueryCapabilityGrantWitnessBinding {
    pub const fn new(path_index: usize, entity_ordinal: usize) -> Self {
        Self {
            path_index,
            entity_ordinal,
        }
    }

    pub const fn path_index(self) -> usize {
        self.path_index
    }

    pub const fn entity_ordinal(self) -> usize {
        self.entity_ordinal
    }
}

pub struct WorthQueryCapabilityPathTemplate {
    pub plan: RelationalAuthorizationPathPlan,
    pub identity: [u8; 32],
    pub guard: WorthQueryCapabilityRequestGuard,
    pub grant_ordinal: Option<usize>,
    pub elevation_ordinals: Vec<usize>,
    pub elevation_resource_ordinal: Option<usize>,
    pub context_anchors: Vec<WorthQueryCapabilityContextAnchor>,
}

pub enum WorthQueryCapabilityRequestGuard {
    Unconditional,
    Accepted {
        axis: WorthQueryCapabilityRequestValueAxis,
        values: Vec<AspectValue>,
    },
}

#[derive(Clone, Copy)]
pub enum WorthQueryCapabilityRequestValueAxis {
    Action,
    Purpose,
    Field,
    Magnitude,
}

pub struct WorthQueryCapabilityContextAnchor {
    pub ordinal: usize,
    pub kind: KindId,
    pub context: String,
    pub context_type: String,
    pub slot: String,
    pub slot_type: String,
    pub entity: String,
}

pub struct WorthQueryCapabilityRequestBindings {
    pub action: AspectValue,
    pub purpose: AspectValue,
    pub resource_entity: String,
    pub related_relation: Option<RelationalAuthorizationTraversal>,
    pub field: Option<AspectFieldLocator>,
    pub magnitude: Option<AspectFieldLocator>,
    pub cardinality: ApplicationCapabilityCardinalityDimension,
    pub timeline: ApplicationCapabilityValidityTimeline,
    /// The grant's inclusive lower validity bound; `None` is open below.
    pub not_before: Option<AspectFieldLocator>,
    /// The grant's inclusive upper validity bound; `None` never expires.
    pub not_after: Option<AspectFieldLocator>,
    pub context: String,
    pub context_type: String,
}

pub const fn field_binding(
    dimension: &ApplicationCapabilityFieldDimension,
) -> Option<
    &worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldBinding,
> {
    match dimension {
        ApplicationCapabilityFieldDimension::NotApplicable => None,
        ApplicationCapabilityFieldDimension::Bound(binding) => Some(binding),
    }
}

mod court_doors_for_test;
