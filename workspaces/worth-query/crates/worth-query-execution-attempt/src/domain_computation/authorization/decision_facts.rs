use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_relational::facade::authorization::RelationalAuthorizationObservationCounters;
use worth_runtime_bridge::facade::BridgeAuthorizationRuntime;

use super::{
    WorthQueryCapabilityCommitBasis, WorthQueryOperationAdmissionIdentity,
    WorthQueryRetainedCapabilityAuthorization,
};
pub mod admitted_currentness;
pub mod source_read_support;
pub use admitted_currentness::AuthorizationCurrentnessStop;
pub use source_read_support::{
    SourceAuthorizationLocus, SourceReadSupport, SourceReadSupportCurrentnessStop,
};
pub mod authorization;
pub mod principal_currentness;
pub mod provider_binding;

pub use authorization::WorthQueryDelegationActivationDecisionFact;
pub use authorization::WorthQueryDelegationDecisionFact;
pub use authorization::WorthQueryDurableAuthorizationDependencies;
pub use authorization::WorthQueryDurableCapabilityLineage;
pub use principal_currentness::WorthQueryPrincipalCurrentnessDependency;
pub use provider_binding::{
    WorthQueryProviderAuthorizationDecisionFacts, WorthQueryProviderDecisionFactBinding,
};

pub enum WorthQueryRetainedAuthorizationDecisionFacts {
    Principal(WorthQueryPrincipalCurrentnessDependency),
    Abilities {
        principal: WorthQueryPrincipalCurrentnessDependency,
        decisions: Vec<WorthQueryAuthorizationDecisionFact>,
    },
    Capability(WorthQueryRetainedCapabilityAuthorization),
}

impl WorthQueryRetainedAuthorizationDecisionFacts {
    pub fn signing_principal(
        &self,
    ) -> &std::sync::Arc<
        worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
    > {
        principal(self).external_identity()
    }
    pub const fn principal(principal: WorthQueryPrincipalCurrentnessDependency) -> Self {
        Self::Principal(principal)
    }

    pub fn abilities(
        principal: WorthQueryPrincipalCurrentnessDependency,
        decisions: Vec<WorthQueryAuthorizationDecisionFact>,
    ) -> Self {
        Self::Abilities {
            principal,
            decisions,
        }
    }

    pub const fn capability(authorization: WorthQueryRetainedCapabilityAuthorization) -> Self {
        Self::Capability(authorization)
    }

    pub const fn capability_authorization(
        &self,
    ) -> Option<&WorthQueryRetainedCapabilityAuthorization> {
        match self {
            Self::Capability(authorization) => Some(authorization),
            Self::Principal(_) | Self::Abilities { .. } => None,
        }
    }

    pub const fn capability_authorization_mut(
        &mut self,
    ) -> Option<&mut WorthQueryRetainedCapabilityAuthorization> {
        match self {
            Self::Capability(authorization) => Some(authorization),
            Self::Principal(_) | Self::Abilities { .. } => None,
        }
    }

    pub(super) fn policy_count(&self) -> usize {
        match self {
            Self::Principal(_) => 0,
            Self::Abilities { decisions, .. } => decisions.len(),
            Self::Capability(authorization) => authorization.exact_fact_count() - 1,
        }
    }

    pub fn exact_fact_count(&self) -> usize {
        1usize.saturating_add(self.policy_count())
    }

    pub fn belongs_to_session(
        &self,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        match self {
            Self::Capability(authorization) => authorization.belongs_to_session(session),
            Self::Principal(_) | Self::Abilities { .. } => {
                principal(self).session_identity() == session
                    && decisions(self).all(|fact| fact.session_identity() == session)
            }
        }
    }

    pub(super) fn relational_counters(&self) -> RelationalAuthorizationObservationCounters {
        if let Self::Capability(authorization) = self {
            return authorization.relational_counters();
        }
        decisions(self).fold(
            RelationalAuthorizationObservationCounters::default(),
            |mut total, fact| {
                let counters = fact.relational_counters();
                total.paths_evaluated += counters.paths_evaluated;
                total.adjacency_lists_read += counters.adjacency_lists_read;
                total.adjacency_edges_inspected += counters.adjacency_edges_inspected;
                total.relation_records_inspected += counters.relation_records_inspected;
                total.entity_records_inspected += counters.entity_records_inspected;
                total.predicate_fields_inspected += counters.predicate_fields_inspected;
                total.maximum_frontier_width = total
                    .maximum_frontier_width
                    .max(counters.maximum_frontier_width);
                total
            },
        )
    }

    pub(super) fn signal_dependency_count(&self) -> usize {
        match self {
            Self::Capability(authorization) => authorization.signal_dependency_count(),
            Self::Principal(_) | Self::Abilities { .. } => decisions(self)
                .map(|fact| fact.signal_dependency_count())
                .sum(),
        }
    }

    pub(super) fn bridge_is_retained(&self, bridge: &BridgeAuthorizationRuntime) -> bool {
        match self {
            Self::Capability(authorization) => authorization.bridge_is_retained(bridge),
            Self::Principal(_) | Self::Abilities { .. } => {
                decisions(self).all(|fact| fact.bridge_is_retained(bridge))
            }
        }
    }

    pub fn validate_currentness_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &BridgeAuthorizationRuntime,
    ) -> Result<(), super::WorthQueryOperationAuthorizationDenialKind> {
        if !self.principal_remains_current_in(execution, runtime, snapshot)? {
            return Err(super::WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::StalePrincipal,
            ));
        }
        self.decisions_remain_current_in(execution, runtime, snapshot, bridge)?
            .then_some(())
            .ok_or(super::WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::StaleAuthorization,
            ))
    }

    pub(super) fn principal_remains_current_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<bool, super::WorthQueryOperationAuthorizationDenialKind> {
        principal(self).remains_current_in(execution, runtime, snapshot)
    }

    pub(super) fn decisions_remain_current_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &BridgeAuthorizationRuntime,
    ) -> Result<bool, super::WorthQueryOperationAuthorizationDenialKind> {
        match self {
            Self::Capability(authorization) => {
                match authorization.validate_currentness_in(execution, runtime, snapshot, bridge) {
                    Ok(()) => Ok(true),
                    Err(kind @ super::WorthQueryOperationAuthorizationDenialKind::Stopped(_)) => {
                        Err(kind)
                    }
                    Err(super::WorthQueryOperationAuthorizationDenialKind::Refused(_)) => Ok(false),
                }
            }
            Self::Principal(_) | Self::Abilities { .. } => {
                for fact in decisions(self) {
                    if !fact
                        .remains_current_in(execution, runtime, snapshot, bridge)
                        .map_err(|cause| {
                            super::denial::relational_read_denial(
                                cause,
                                "authorization currentness",
                            )
                            .kind()
                        })?
                    {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }

    pub fn into_provider_commit_authorization(
        self,
        admission_identity: WorthQueryOperationAdmissionIdentity,
    ) -> WorthQueryProviderCommitAuthorization {
        match self {
            Self::Principal(principal) => {
                let commit = WorthQueryObservedCommitBasis::new(principal.clone(), Vec::new());
                WorthQueryProviderCommitAuthorization::new(
                    WorthQueryProviderAuthorizationDecisionFacts::new(principal, Vec::new()),
                    WorthQueryCommitAuthorizationBasis::Observed {
                        admission_identity,
                        authorization: commit,
                    },
                )
            }
            Self::Abilities {
                principal,
                decisions,
            } => {
                let commit =
                    WorthQueryObservedCommitBasis::new(principal.clone(), decisions.clone());
                WorthQueryProviderCommitAuthorization::new(
                    WorthQueryProviderAuthorizationDecisionFacts::new(principal, decisions),
                    WorthQueryCommitAuthorizationBasis::Observed {
                        admission_identity,
                        authorization: commit,
                    },
                )
            }
            Self::Capability(authorization) => {
                authorization.into_provider_commit_authorization(admission_identity)
            }
        }
    }
}

fn principal(
    facts: &WorthQueryRetainedAuthorizationDecisionFacts,
) -> &WorthQueryPrincipalCurrentnessDependency {
    match facts {
        WorthQueryRetainedAuthorizationDecisionFacts::Principal(principal)
        | WorthQueryRetainedAuthorizationDecisionFacts::Abilities { principal, .. } => principal,
        WorthQueryRetainedAuthorizationDecisionFacts::Capability(authorization) => {
            authorization.principal()
        }
    }
}

fn decisions(
    facts: &WorthQueryRetainedAuthorizationDecisionFacts,
) -> impl Iterator<Item = &WorthQueryAuthorizationDecisionFact> {
    let slice = match facts {
        WorthQueryRetainedAuthorizationDecisionFacts::Principal(_) => &[][..],
        WorthQueryRetainedAuthorizationDecisionFacts::Abilities { decisions, .. } => decisions,
        WorthQueryRetainedAuthorizationDecisionFacts::Capability(authorization) => {
            std::slice::from_ref(authorization.decision())
        }
    };
    slice.iter()
}

pub enum WorthQueryCommitAuthorizationBasis {
    Observed {
        admission_identity: WorthQueryOperationAdmissionIdentity,
        authorization: WorthQueryObservedCommitBasis,
    },
    Capability {
        admission_identity: WorthQueryOperationAdmissionIdentity,
        authorization: WorthQueryCapabilityCommitBasis,
    },
}

impl WorthQueryCommitAuthorizationBasis {
    pub const fn admission_identity(&self) -> WorthQueryOperationAdmissionIdentity {
        match self {
            Self::Observed {
                admission_identity, ..
            }
            | Self::Capability {
                admission_identity, ..
            } => *admission_identity,
        }
    }

    pub fn belongs_to_session(
        &self,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        match self {
            Self::Observed { authorization, .. } => authorization.belongs_to_session(session),
            Self::Capability { authorization, .. } => authorization.belongs_to_session(session),
        }
    }
}

pub struct WorthQueryObservedCommitBasis {
    principal: WorthQueryPrincipalCurrentnessDependency,
    decisions: Vec<WorthQueryAuthorizationDecisionFact>,
}

impl WorthQueryObservedCommitBasis {
    fn new(
        principal: WorthQueryPrincipalCurrentnessDependency,
        decisions: Vec<WorthQueryAuthorizationDecisionFact>,
    ) -> Self {
        Self {
            principal,
            decisions,
        }
    }

    fn belongs_to_session(
        &self,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        self.principal.session_identity() == session
            && self
                .decisions
                .iter()
                .all(|fact| fact.session_identity() == session)
    }

    pub(super) fn principal_remains_current_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<bool, super::WorthQueryOperationAuthorizationDenialKind> {
        self.principal
            .remains_current_in(execution, runtime, snapshot)
    }

    pub(super) fn decisions_remain_current_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &BridgeAuthorizationRuntime,
    ) -> Result<bool, super::WorthQueryOperationAuthorizationDenialKind> {
        for fact in &self.decisions {
            if !fact
                .remains_current_in(execution, runtime, snapshot, bridge)
                .map_err(|cause| {
                    super::denial::relational_read_denial(cause, "authorization currentness").kind()
                })?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

pub use authorization::WorthQueryAuthorizationDecisionFact;
pub mod provider_commit_authorization;
pub use provider_commit_authorization::{
    WorthQueryProviderCommitAuthorization, WorthQueryRegisteredCommitAuthorization,
};
