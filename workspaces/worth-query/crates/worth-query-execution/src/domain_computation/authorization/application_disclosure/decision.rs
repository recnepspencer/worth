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
mod inspection;
mod preparation;
pub(in crate::domain_computation) use preparation::WorthQueryPreparedApplicationGovernance;
mod resource;
pub(in crate::domain_computation) use inspection::prepare_application_inspection_governance;
pub(in crate::domain_computation) use resource::admit_application_resource_governance;

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
    preparation::prepare(contract, pending).map(|prepared| prepared.bind(binding))
}

mod binding;
pub(in crate::domain_computation) use binding::WorthQueryApplicationGovernanceBinding;
