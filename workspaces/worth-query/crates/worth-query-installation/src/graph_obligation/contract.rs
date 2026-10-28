use std::sync::Arc;

use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::application_capability::ErasedApplicationCapabilityContract;

use crate::application_capability::WorthQueryInstalledApplicationCapabilityIdentity;
use crate::application_operation::WorthQueryInstalledAbilityRequirement;
use crate::application_query::WorthQueryInstalledGraphReadContract;
use crate::domain_computation::WorthQueryExecutionResourceContract;
use crate::domain_operation::{
    WorthQueryInstalledInvariantExecutionRequirement, WorthQueryOperationEffectFamily,
    WorthQueryOperationGraphReadRole, WorthQueryOperationTouchContract,
};

use super::owner::{APPLICATION_TOUCH, INSTALLED_INVARIANT, POLICY_EVALUATION, RELATIONAL_GRAPH};
use super::{WorthQueryInstalledGraphObligationKind, WorthQueryInstalledGraphObligationOwner};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledGraphCapabilityRequirement {
    identity: WorthQueryInstalledApplicationCapabilityIdentity,
    contract: ErasedApplicationCapabilityContract,
}

impl WorthQueryInstalledGraphCapabilityRequirement {
    pub(crate) fn new(
        identity: WorthQueryInstalledApplicationCapabilityIdentity,
        contract: ErasedApplicationCapabilityContract,
    ) -> Self {
        Self { identity, contract }
    }

    pub const fn identity(&self) -> &WorthQueryInstalledApplicationCapabilityIdentity {
        &self.identity
    }

    pub const fn contract(&self) -> &ErasedApplicationCapabilityContract {
        &self.contract
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledGraphObligationResourcePosture {
    ApplicationInspection {
        meaning: CanonicalDigestId,
        maximum_authorization_facts: usize,
    },
    ApplicationResource {
        maximum_authorization_facts: usize,
    },
    ApplicationQuery {
        maximum_traversal_depth: usize,
        maximum_result_count: usize,
        maximum_authorization_facts: usize,
    },
    ApplicationOperation(WorthQueryExecutionResourceContract),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledGraphObligationEffectPosture {
    Observational,
    Policy,
    Mutating,
    Invariant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledGraphObligationTerminalRequirement {
    GraphReadProduct,
    AuthorizationDecisionFact,
    TouchedScopeEvidence,
    EffectApplicationReceipt,
    InvariantVerdict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledGraphObligationSelectionBasis<'a> {
    ApplicationQueryGraph(&'a WorthQueryInstalledGraphReadContract),
    ApplicationOperationGraphRole(&'a WorthQueryOperationGraphReadRole),
    AuthenticatedAccessContext,
    MutationTouch(&'a WorthQueryOperationTouchContract),
    ProposedState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledGraphAuthorizationRequirement<'a> {
    Principal,
    Abilities(&'a [WorthQueryInstalledAbilityRequirement]),
    Capabilities(&'a [WorthQueryInstalledGraphCapabilityRequirement]),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledGraphObligationIdentity {
    set_identity: CanonicalDigestId,
    slot: u32,
}

impl WorthQueryInstalledGraphObligationIdentity {
    pub(super) const fn new(set_identity: CanonicalDigestId, slot: u32) -> Self {
        Self { set_identity, slot }
    }

    pub const fn set_identity(&self) -> &CanonicalDigestId {
        &self.set_identity
    }

    pub const fn slot(&self) -> u32 {
        self.slot
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryInstalledGraphObligationContract {
    QueryGraphRead {
        graph: WorthQueryInstalledGraphReadContract,
    },
    OperationGraphRead {
        role: WorthQueryOperationGraphReadRole,
    },
    PrincipalAuthorization,
    AbilityAuthorization {
        requirements: Vec<WorthQueryInstalledAbilityRequirement>,
    },
    CapabilityAuthorization {
        requirements: Vec<WorthQueryInstalledGraphCapabilityRequirement>,
    },
    MutationTouch {
        contract: WorthQueryOperationTouchContract,
    },
    EffectApplication {
        family: WorthQueryOperationEffectFamily,
    },
    InvariantExecution {
        requirement: WorthQueryInstalledInvariantExecutionRequirement,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledGraphObligation {
    identity: WorthQueryInstalledGraphObligationIdentity,
    contract: WorthQueryInstalledGraphObligationContract,
    resources: Arc<WorthQueryInstalledGraphObligationResourcePosture>,
}

impl WorthQueryInstalledGraphObligation {
    pub(super) fn new(
        identity: WorthQueryInstalledGraphObligationIdentity,
        contract: WorthQueryInstalledGraphObligationContract,
        resources: Arc<WorthQueryInstalledGraphObligationResourcePosture>,
    ) -> Self {
        Self {
            identity,
            contract,
            resources,
        }
    }

    pub const fn identity(&self) -> &WorthQueryInstalledGraphObligationIdentity {
        &self.identity
    }

    pub const fn kind(&self) -> WorthQueryInstalledGraphObligationKind {
        match self.contract {
            WorthQueryInstalledGraphObligationContract::QueryGraphRead { .. }
            | WorthQueryInstalledGraphObligationContract::OperationGraphRead { .. } => {
                WorthQueryInstalledGraphObligationKind::GraphRead
            }
            WorthQueryInstalledGraphObligationContract::PrincipalAuthorization
            | WorthQueryInstalledGraphObligationContract::AbilityAuthorization { .. }
            | WorthQueryInstalledGraphObligationContract::CapabilityAuthorization { .. } => {
                WorthQueryInstalledGraphObligationKind::AuthorizationObservation
            }
            WorthQueryInstalledGraphObligationContract::MutationTouch { .. } => {
                WorthQueryInstalledGraphObligationKind::MutationTouch
            }
            WorthQueryInstalledGraphObligationContract::EffectApplication { .. } => {
                WorthQueryInstalledGraphObligationKind::EffectApplication
            }
            WorthQueryInstalledGraphObligationContract::InvariantExecution { .. } => {
                WorthQueryInstalledGraphObligationKind::InvariantExecution
            }
        }
    }

    pub const fn required_owners(&self) -> &'static [WorthQueryInstalledGraphObligationOwner] {
        match self.contract {
            WorthQueryInstalledGraphObligationContract::QueryGraphRead { .. }
            | WorthQueryInstalledGraphObligationContract::OperationGraphRead { .. }
            | WorthQueryInstalledGraphObligationContract::PrincipalAuthorization
            | WorthQueryInstalledGraphObligationContract::EffectApplication { .. } => {
                RELATIONAL_GRAPH
            }
            WorthQueryInstalledGraphObligationContract::AbilityAuthorization { .. }
            | WorthQueryInstalledGraphObligationContract::CapabilityAuthorization { .. } => {
                POLICY_EVALUATION
            }
            WorthQueryInstalledGraphObligationContract::MutationTouch { .. } => APPLICATION_TOUCH,
            WorthQueryInstalledGraphObligationContract::InvariantExecution { .. } => {
                INSTALLED_INVARIANT
            }
        }
    }

    pub const fn selection_basis(&self) -> WorthQueryInstalledGraphObligationSelectionBasis<'_> {
        match &self.contract {
            WorthQueryInstalledGraphObligationContract::QueryGraphRead { graph } => {
                WorthQueryInstalledGraphObligationSelectionBasis::ApplicationQueryGraph(graph)
            }
            WorthQueryInstalledGraphObligationContract::OperationGraphRead { role } => {
                WorthQueryInstalledGraphObligationSelectionBasis::ApplicationOperationGraphRole(
                    role,
                )
            }
            WorthQueryInstalledGraphObligationContract::PrincipalAuthorization
            | WorthQueryInstalledGraphObligationContract::AbilityAuthorization { .. }
            | WorthQueryInstalledGraphObligationContract::CapabilityAuthorization { .. } => {
                WorthQueryInstalledGraphObligationSelectionBasis::AuthenticatedAccessContext
            }
            WorthQueryInstalledGraphObligationContract::MutationTouch { contract } => {
                WorthQueryInstalledGraphObligationSelectionBasis::MutationTouch(contract)
            }
            WorthQueryInstalledGraphObligationContract::EffectApplication { .. }
            | WorthQueryInstalledGraphObligationContract::InvariantExecution { .. } => {
                WorthQueryInstalledGraphObligationSelectionBasis::ProposedState
            }
        }
    }

    pub fn resource_posture(&self) -> &WorthQueryInstalledGraphObligationResourcePosture {
        &self.resources
    }

    pub const fn effect_posture(&self) -> WorthQueryInstalledGraphObligationEffectPosture {
        match self.contract {
            WorthQueryInstalledGraphObligationContract::QueryGraphRead { .. }
            | WorthQueryInstalledGraphObligationContract::OperationGraphRead { .. }
            | WorthQueryInstalledGraphObligationContract::MutationTouch { .. } => {
                WorthQueryInstalledGraphObligationEffectPosture::Observational
            }
            WorthQueryInstalledGraphObligationContract::PrincipalAuthorization
            | WorthQueryInstalledGraphObligationContract::AbilityAuthorization { .. }
            | WorthQueryInstalledGraphObligationContract::CapabilityAuthorization { .. } => {
                WorthQueryInstalledGraphObligationEffectPosture::Policy
            }
            WorthQueryInstalledGraphObligationContract::EffectApplication { .. } => {
                WorthQueryInstalledGraphObligationEffectPosture::Mutating
            }
            WorthQueryInstalledGraphObligationContract::InvariantExecution { .. } => {
                WorthQueryInstalledGraphObligationEffectPosture::Invariant
            }
        }
    }

    pub const fn terminal_requirement(
        &self,
    ) -> WorthQueryInstalledGraphObligationTerminalRequirement {
        match self.contract {
            WorthQueryInstalledGraphObligationContract::QueryGraphRead { .. }
            | WorthQueryInstalledGraphObligationContract::OperationGraphRead { .. } => {
                WorthQueryInstalledGraphObligationTerminalRequirement::GraphReadProduct
            }
            WorthQueryInstalledGraphObligationContract::PrincipalAuthorization
            | WorthQueryInstalledGraphObligationContract::AbilityAuthorization { .. }
            | WorthQueryInstalledGraphObligationContract::CapabilityAuthorization { .. } => {
                WorthQueryInstalledGraphObligationTerminalRequirement::AuthorizationDecisionFact
            }
            WorthQueryInstalledGraphObligationContract::MutationTouch { .. } => {
                WorthQueryInstalledGraphObligationTerminalRequirement::TouchedScopeEvidence
            }
            WorthQueryInstalledGraphObligationContract::EffectApplication { .. } => {
                WorthQueryInstalledGraphObligationTerminalRequirement::EffectApplicationReceipt
            }
            WorthQueryInstalledGraphObligationContract::InvariantExecution { .. } => {
                WorthQueryInstalledGraphObligationTerminalRequirement::InvariantVerdict
            }
        }
    }

    pub fn authorization_requirement(
        &self,
    ) -> Option<WorthQueryInstalledGraphAuthorizationRequirement<'_>> {
        match &self.contract {
            WorthQueryInstalledGraphObligationContract::PrincipalAuthorization => {
                Some(WorthQueryInstalledGraphAuthorizationRequirement::Principal)
            }
            WorthQueryInstalledGraphObligationContract::AbilityAuthorization { requirements } => {
                Some(WorthQueryInstalledGraphAuthorizationRequirement::Abilities(
                    requirements,
                ))
            }
            WorthQueryInstalledGraphObligationContract::CapabilityAuthorization {
                requirements,
            } => Some(WorthQueryInstalledGraphAuthorizationRequirement::Capabilities(requirements)),
            _ => None,
        }
    }

    pub const fn effect_family(&self) -> Option<WorthQueryOperationEffectFamily> {
        match self.contract {
            WorthQueryInstalledGraphObligationContract::EffectApplication { family } => {
                Some(family)
            }
            _ => None,
        }
    }

    pub const fn invariant_requirement(
        &self,
    ) -> Option<&WorthQueryInstalledInvariantExecutionRequirement> {
        match &self.contract {
            WorthQueryInstalledGraphObligationContract::InvariantExecution { requirement } => {
                Some(requirement)
            }
            _ => None,
        }
    }
}
