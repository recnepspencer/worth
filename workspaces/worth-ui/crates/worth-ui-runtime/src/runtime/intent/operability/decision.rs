use super::{
    UiIntentAffinityPosture, UiIntentConfirmationPosture, UiIntentMutabilityPosture,
    UiIntentOccupancyPosture, UiIntentPolicyPosture, UiIntentReadinessPosture,
    UiIntentSupportPosture, UiIntentWithheldCondition,
};
use crate::declaration::UiIntentOperabilityDependencyAxis;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiIntentInoperableCause {
    Unsupported,
    WrongWorld,
    RebindRequired,
    StaleTarget,
    /// A condition source of `axis` holds no truth value. It ranks below
    /// target and world causes and above every posture it could be mistaken
    /// for, so a withheld policy never reads as a denied one.
    ConditionWithheld {
        axis: UiIntentOperabilityDependencyAxis,
        condition: UiIntentWithheldCondition,
    },
    PolicyDenied,
    Occupied,
    Readonly,
    Pending,
    ConfirmationRequired {
        policy_identity: Box<str>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiIntentOperabilityCost {
    selected_dependencies_visited: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiIntentOperabilityDecision {
    contract_identity: Box<str>,
    support: UiIntentSupportPosture,
    mutability: UiIntentMutabilityPosture,
    readiness: UiIntentReadinessPosture,
    occupancy: UiIntentOccupancyPosture,
    policy: UiIntentPolicyPosture,
    affinity: UiIntentAffinityPosture,
    confirmation: UiIntentConfirmationPosture,
    cost: UiIntentOperabilityCost,
}

pub struct UiIntentInoperableCauseIter<'decision> {
    decision: &'decision UiIntentOperabilityDecision,
    next_priority: u8,
}

pub(crate) struct UiIntentOperabilityDecisionInput {
    pub(crate) contract_identity: Box<str>,
    pub(crate) support: UiIntentSupportPosture,
    pub(crate) mutability: UiIntentMutabilityPosture,
    pub(crate) readiness: UiIntentReadinessPosture,
    pub(crate) occupancy: UiIntentOccupancyPosture,
    pub(crate) policy: UiIntentPolicyPosture,
    pub(crate) affinity: UiIntentAffinityPosture,
    pub(crate) confirmation: UiIntentConfirmationPosture,
    pub(crate) selected_dependencies_visited: usize,
}

impl UiIntentOperabilityDecision {
    pub(crate) fn new(input: UiIntentOperabilityDecisionInput) -> Self {
        Self {
            contract_identity: input.contract_identity,
            support: input.support,
            mutability: input.mutability,
            readiness: input.readiness,
            occupancy: input.occupancy,
            policy: input.policy,
            affinity: input.affinity,
            confirmation: input.confirmation,
            cost: UiIntentOperabilityCost {
                selected_dependencies_visited: input.selected_dependencies_visited,
            },
        }
    }

    pub fn contract_identity(&self) -> &str {
        &self.contract_identity
    }

    pub const fn support(&self) -> UiIntentSupportPosture {
        self.support
    }

    pub const fn mutability(&self) -> UiIntentMutabilityPosture {
        self.mutability
    }

    pub const fn readiness(&self) -> UiIntentReadinessPosture {
        self.readiness
    }

    pub const fn occupancy(&self) -> UiIntentOccupancyPosture {
        self.occupancy
    }

    pub const fn policy(&self) -> UiIntentPolicyPosture {
        self.policy
    }

    pub const fn affinity(&self) -> UiIntentAffinityPosture {
        self.affinity
    }

    pub const fn confirmation(&self) -> &UiIntentConfirmationPosture {
        &self.confirmation
    }

    pub const fn cost(&self) -> UiIntentOperabilityCost {
        self.cost
    }

    pub fn is_operable(&self) -> bool {
        self.primary_cause().is_none()
    }

    pub fn causes(&self) -> UiIntentInoperableCauseIter<'_> {
        UiIntentInoperableCauseIter {
            decision: self,
            next_priority: 0,
        }
    }

    pub fn primary_cause(&self) -> Option<UiIntentInoperableCause> {
        self.causes().next()
    }

    #[cfg(test)]
    pub(crate) fn with_affinity_for_test(&self, affinity: UiIntentAffinityPosture) -> Self {
        Self {
            affinity,
            ..self.clone()
        }
    }

    #[cfg(test)]
    pub(crate) fn ready_for_test() -> Self {
        Self::new(UiIntentOperabilityDecisionInput {
            contract_identity: "appearance-state-test-intent".into(),
            support: UiIntentSupportPosture::Supported,
            mutability: UiIntentMutabilityPosture::Writable,
            readiness: UiIntentReadinessPosture::Ready,
            occupancy: UiIntentOccupancyPosture::Idle,
            policy: UiIntentPolicyPosture::Admitted,
            affinity: UiIntentAffinityPosture::Current,
            confirmation: UiIntentConfirmationPosture::NotRequired,
            selected_dependencies_visited: 0,
        })
    }
}

impl UiIntentOperabilityCost {
    pub const fn selected_dependencies_visited(self) -> usize {
        self.selected_dependencies_visited
    }
}

impl Iterator for UiIntentInoperableCauseIter<'_> {
    type Item = UiIntentInoperableCause;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_priority < CAUSE_PRIORITIES {
            let priority = self.next_priority;
            self.next_priority += 1;
            if let Some(cause) = cause_at(self.decision, priority) {
                return Some(cause);
            }
        }
        None
    }
}

const CAUSE_PRIORITIES: u8 = 12;

fn cause_at(
    decision: &UiIntentOperabilityDecision,
    priority: u8,
) -> Option<UiIntentInoperableCause> {
    let withheld = |axis, posture: Option<UiIntentWithheldCondition>| {
        posture.map(|condition| UiIntentInoperableCause::ConditionWithheld { axis, condition })
    };
    match priority {
        0 if decision.support == UiIntentSupportPosture::Unsupported => {
            Some(UiIntentInoperableCause::Unsupported)
        }
        1 if decision.affinity == UiIntentAffinityPosture::WrongWorld => {
            Some(UiIntentInoperableCause::WrongWorld)
        }
        2 if decision.affinity == UiIntentAffinityPosture::RebindRequired => {
            Some(UiIntentInoperableCause::RebindRequired)
        }
        3 if decision.affinity == UiIntentAffinityPosture::Stale => {
            Some(UiIntentInoperableCause::StaleTarget)
        }
        4 => withheld(
            UiIntentOperabilityDependencyAxis::Policy,
            match decision.policy {
                UiIntentPolicyPosture::Withheld(condition) => Some(condition),
                UiIntentPolicyPosture::Admitted | UiIntentPolicyPosture::Denied => None,
            },
        ),
        5 => withheld(
            UiIntentOperabilityDependencyAxis::Mutability,
            match decision.mutability {
                UiIntentMutabilityPosture::Withheld(condition) => Some(condition),
                UiIntentMutabilityPosture::Writable | UiIntentMutabilityPosture::Readonly => None,
            },
        ),
        6 => withheld(
            UiIntentOperabilityDependencyAxis::Readiness,
            match decision.readiness {
                UiIntentReadinessPosture::Withheld(condition) => Some(condition),
                UiIntentReadinessPosture::Ready | UiIntentReadinessPosture::Pending => None,
            },
        ),
        7 if decision.policy == UiIntentPolicyPosture::Denied => {
            Some(UiIntentInoperableCause::PolicyDenied)
        }
        8 if decision.occupancy == UiIntentOccupancyPosture::InFlight => {
            Some(UiIntentInoperableCause::Occupied)
        }
        9 if decision.mutability == UiIntentMutabilityPosture::Readonly => {
            Some(UiIntentInoperableCause::Readonly)
        }
        10 if decision.readiness == UiIntentReadinessPosture::Pending => {
            Some(UiIntentInoperableCause::Pending)
        }
        11 => decision
            .confirmation
            .required_policy_identity()
            .map(|identity| UiIntentInoperableCause::ConfirmationRequired {
                policy_identity: identity.into(),
            }),
        _ => None,
    }
}
