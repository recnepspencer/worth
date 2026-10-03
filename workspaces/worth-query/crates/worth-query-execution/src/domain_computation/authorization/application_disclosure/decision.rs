use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, CanonicalDigestId};
use worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey;
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity;
use worth_relational::facade::{history::BranchId, identity::EntityId};

use super::contract::{AdmittedDisclosureRule, WorthQueryAdmittedApplicationDisclosureContract};
use super::influence_validation::GovernedInternalFieldRules;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::provider_session::{
    WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity,
    WorthQueryManagedGraphWorkSession,
};

use super::super::WorthQueryRetainedCapabilityAuthorization;

mod field_admission;
mod governance;

pub(in crate::domain_computation) use field_admission::{
    WorthQueryApplicationDisclosedProjectionAdmission,
    WorthQueryApplicationInternalProjectionAdmission,
};

pub(in crate::domain_computation) struct WorthQueryPendingApplicationQueryGovernance {
    capability_name: String,
    capability_type: String,
    disclosure_value: AspectValue,
    authorization_canonical_work: WorthQueryCanonicalWorkEvidence,
    authorization: WorthQueryRetainedCapabilityAuthorization,
}

impl WorthQueryPendingApplicationQueryGovernance {
    pub(in crate::domain_computation) fn new(
        capability_name: impl Into<String>,
        capability_type: impl Into<String>,
        disclosure_value: AspectValue,
        authorization_canonical_work: WorthQueryCanonicalWorkEvidence,
        authorization: WorthQueryRetainedCapabilityAuthorization,
    ) -> Self {
        Self {
            capability_name: capability_name.into(),
            capability_type: capability_type.into(),
            disclosure_value,
            authorization_canonical_work,
            authorization,
        }
    }

    pub(in crate::domain_computation) fn authorization_mut(
        &mut self,
    ) -> &mut WorthQueryRetainedCapabilityAuthorization {
        &mut self.authorization
    }

    pub(in crate::domain_computation) fn installed_capability_identity(&self) -> [u8; 32] {
        self.authorization.installed_capability_identity()
    }
}

pub(in crate::domain_computation) struct WorthQueryApplicationInternalComputationAuthority {
    binding: WorthQueryApplicationGovernanceBinding,
    internal_field_rules: GovernedInternalFieldRules,
    disclosure_value: AspectValue,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorthQueryApplicationDisclosureDecision {
    classification: String,
    disclosed: BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
    omitted: BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
}

pub(in crate::domain_computation) enum WorthQueryApplicationQueryGovernance {
    Public,
    Governed {
        capability_name: String,
        capability_type: String,
        disclosure_value: AspectValue,
        computation: WorthQueryApplicationInternalComputationAuthority,
        disclosure: WorthQueryApplicationDisclosureDecision,
        authorization: WorthQueryRetainedCapabilityAuthorization,
        authorization_canonical_work: WorthQueryCanonicalWorkEvidence,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryApplicationQueryGovernanceDenialKind {
    Required,
    CapabilityMismatch,
    InternalComputationDenied,
}

pub(in crate::domain_computation) fn admit_application_query_governance(
    contract: WorthQueryAdmittedApplicationDisclosureContract,
    pending: Option<WorthQueryPendingApplicationQueryGovernance>,
    binding: WorthQueryApplicationGovernanceBinding,
) -> Result<WorthQueryApplicationQueryGovernance, WorthQueryApplicationQueryGovernanceDenialKind> {
    match (contract, pending) {
        (WorthQueryAdmittedApplicationDisclosureContract::Public, None) => {
            Ok(WorthQueryApplicationQueryGovernance::Public)
        }
        (
            WorthQueryAdmittedApplicationDisclosureContract::Governed {
                classification,
                capability_name,
                capability_type,
                result_rules,
                internal_rules,
                internal_field_rules,
            },
            Some(pending),
        ) => {
            if capability_name != pending.capability_name
                || capability_type != pending.capability_type
            {
                return Err(WorthQueryApplicationQueryGovernanceDenialKind::CapabilityMismatch);
            }
            if internal_rules
                .iter()
                .any(|rule| rule.disclosure_value() != &pending.disclosure_value)
            {
                return Err(
                    WorthQueryApplicationQueryGovernanceDenialKind::InternalComputationDenied,
                );
            }
            let (disclosed, omitted) = result_rules.into_iter().fold(
                (BTreeMap::new(), BTreeMap::new()),
                |(mut disclosed, mut omitted), (slot, rule)| {
                    let target = if rule.disclosure_value() == &pending.disclosure_value {
                        &mut disclosed
                    } else {
                        &mut omitted
                    };
                    target.insert(slot, rule);
                    (disclosed, omitted)
                },
            );
            Ok(WorthQueryApplicationQueryGovernance::Governed {
                capability_name: pending.capability_name,
                capability_type: pending.capability_type,
                disclosure_value: pending.disclosure_value.clone(),
                authorization_canonical_work: pending.authorization_canonical_work,
                computation: WorthQueryApplicationInternalComputationAuthority {
                    binding,
                    internal_field_rules,
                    disclosure_value: pending.disclosure_value,
                },
                disclosure: WorthQueryApplicationDisclosureDecision {
                    classification,
                    disclosed,
                    omitted,
                },
                authorization: pending.authorization,
            })
        }
        _ => Err(WorthQueryApplicationQueryGovernanceDenialKind::Required),
    }
}

pub(in crate::domain_computation) struct WorthQueryApplicationGovernanceBinding {
    runtime: WorthQueryRuntimeAuthorityIdentity,
    query: WorthQueryInstalledApplicationQueryIdentity,
    parameters: CanonicalDigestId,
    principal: EntityId,
    scope: EntityId,
    session: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    branch: BranchId,
    basis: crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity,
    provider: String,
}

impl WorthQueryApplicationGovernanceBinding {
    pub(in crate::domain_computation) fn from_session(
        graph_work: &WorthQueryManagedGraphWorkSession,
        query: WorthQueryInstalledApplicationQueryIdentity,
        parameters: CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> Self {
        Self {
            runtime: graph_work.runtime_authority(),
            query,
            parameters,
            principal,
            scope,
            session: graph_work.identity(),
            managed_run: graph_work.managed_run_identity(),
            branch: graph_work.branch().relational().clone(),
            basis: graph_work
                .query_basis()
                .expect("application disclosure is admitted only for a query session")
                .clone(),
            provider: graph_work.provider().to_string(),
        }
    }

    fn matches(
        &self,
        graph_work: &WorthQueryManagedGraphWorkSession,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        self.runtime == runtime
            && self.runtime == graph_work.runtime_authority()
            && &self.query == query
            && &self.parameters == parameters
            && self.principal == principal
            && self.scope == scope
            && self.session == graph_work.identity()
            && self.managed_run == graph_work.managed_run_identity()
            && self.branch == *graph_work.branch().relational()
            && graph_work.query_basis() == Some(&self.basis)
            && self.provider == graph_work.provider()
    }
}
