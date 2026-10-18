use super::{
    ApplicationCapabilityDelegationRule, ApplicationCapabilityDisclosureRule,
    ApplicationCapabilityGraphRule,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityAllowRule(ApplicationCapabilityGraphRule);

impl ApplicationCapabilityAllowRule {
    pub const fn new(rule: ApplicationCapabilityGraphRule) -> Self {
        Self(rule)
    }

    pub const fn graph(&self) -> &ApplicationCapabilityGraphRule {
        &self.0
    }
}

macro_rules! optional_graph_rule {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
        pub enum $name {
            NotApplicable,
            When(ApplicationCapabilityGraphRule),
        }

        impl $name {
            pub const fn not_applicable() -> Self {
                Self::NotApplicable
            }

            pub const fn when(rule: ApplicationCapabilityGraphRule) -> Self {
                Self::When(rule)
            }

            pub const fn graph(&self) -> Option<&ApplicationCapabilityGraphRule> {
                match self {
                    Self::NotApplicable => None,
                    Self::When(rule) => Some(rule),
                }
            }
        }
    };
}

optional_graph_rule!(ApplicationCapabilityDenyRule);
optional_graph_rule!(ApplicationCapabilityConflictRule);
optional_graph_rule!(ApplicationCapabilitySeparationOfDutyRule);
optional_graph_rule!(ApplicationCapabilityDistinctActorRule);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityDecisionComposition {
    allow: ApplicationCapabilityAllowRule,
    deny: ApplicationCapabilityDenyRule,
    conflict: ApplicationCapabilityConflictRule,
}

impl ApplicationCapabilityDecisionComposition {
    pub const fn new(
        allow: ApplicationCapabilityAllowRule,
        deny: ApplicationCapabilityDenyRule,
        conflict: ApplicationCapabilityConflictRule,
    ) -> Self {
        Self {
            allow,
            deny,
            conflict,
        }
    }

    pub const fn allow(&self) -> &ApplicationCapabilityAllowRule {
        &self.allow
    }

    pub const fn deny(&self) -> &ApplicationCapabilityDenyRule {
        &self.deny
    }

    pub const fn conflict(&self) -> &ApplicationCapabilityConflictRule {
        &self.conflict
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityActorComposition {
    separation_of_duty: ApplicationCapabilitySeparationOfDutyRule,
    distinct_actor: ApplicationCapabilityDistinctActorRule,
    recorded_actor: super::ApplicationCapabilityRecordedActorRule,
}

impl ApplicationCapabilityActorComposition {
    pub const fn new(
        separation_of_duty: ApplicationCapabilitySeparationOfDutyRule,
        distinct_actor: ApplicationCapabilityDistinctActorRule,
        recorded_actor: super::ApplicationCapabilityRecordedActorRule,
    ) -> Self {
        Self {
            separation_of_duty,
            distinct_actor,
            recorded_actor,
        }
    }

    pub const fn recorded_actor(&self) -> &super::ApplicationCapabilityRecordedActorRule {
        &self.recorded_actor
    }

    pub const fn separation_of_duty(&self) -> &ApplicationCapabilitySeparationOfDutyRule {
        &self.separation_of_duty
    }

    pub const fn distinct_actor(&self) -> &ApplicationCapabilityDistinctActorRule {
        &self.distinct_actor
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityPropagationComposition {
    delegation: ApplicationCapabilityDelegationRule,
    disclosure: ApplicationCapabilityDisclosureRule,
}

impl ApplicationCapabilityPropagationComposition {
    pub const fn new(
        delegation: ApplicationCapabilityDelegationRule,
        disclosure: ApplicationCapabilityDisclosureRule,
    ) -> Self {
        Self {
            delegation,
            disclosure,
        }
    }

    pub const fn delegation(&self) -> ApplicationCapabilityDelegationRule {
        self.delegation
    }

    pub const fn disclosure(&self) -> &ApplicationCapabilityDisclosureRule {
        &self.disclosure
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityComposition {
    decision: ApplicationCapabilityDecisionComposition,
    actors: ApplicationCapabilityActorComposition,
    propagation: ApplicationCapabilityPropagationComposition,
}

impl ApplicationCapabilityComposition {
    pub const fn new(
        decision: ApplicationCapabilityDecisionComposition,
        actors: ApplicationCapabilityActorComposition,
        propagation: ApplicationCapabilityPropagationComposition,
    ) -> Self {
        Self {
            decision,
            actors,
            propagation,
        }
    }

    pub const fn decision(&self) -> &ApplicationCapabilityDecisionComposition {
        &self.decision
    }

    pub const fn actors(&self) -> &ApplicationCapabilityActorComposition {
        &self.actors
    }

    pub const fn propagation(&self) -> &ApplicationCapabilityPropagationComposition {
        &self.propagation
    }
}
