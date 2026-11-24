use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, CanonicalDigestId};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryResultSlotKey, RelationDirection,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity;
use worth_relational::facade::{history::BranchId, identity::EntityId};

use super::contract::{AdmittedDisclosureRule, WorthQueryAdmittedApplicationDisclosureContract};
use super::influence_validation::{GovernedInternalFieldRules, GovernedInternalRelationRules};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::provider_session::{
    WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity,
    WorthQueryManagedGraphWorkSession,
};

use super::super::WorthQueryRetainedCapabilityAuthorization;

mod diagnostic;
mod diagnostic_preparation;
pub use diagnostic_preparation::prepare_application_diagnostic_governance;
mod field_admission;
mod governance;
mod influence;
pub use influence::{WorthQueryRetainedQueryInfluence, WorthQueryRetainedQueryInfluenceDenial};
mod inspection;
pub use diagnostic::DiagnosticComparisonAdmission;
mod preparation;
pub use preparation::WorthQueryPreparedApplicationGovernance;
mod resource;
pub use inspection::prepare_application_inspection_governance;
pub use resource::admit_application_resource_governance;

pub use field_admission::{
    WorthQueryApplicationDisclosedProjectionAdmission,
    WorthQueryApplicationInternalProjectionAdmission,
};

pub struct WorthQueryPendingApplicationQueryGovernance {
    capability_name: String,
    capability_type: String,
    disclosure_value: AspectValue,
    authorization_canonical_work: WorthQueryCanonicalWorkEvidence,
    authorization: WorthQueryRetainedCapabilityAuthorization,
}

impl WorthQueryPendingApplicationQueryGovernance {
    pub fn new(
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

    pub fn authorization_mut(&mut self) -> &mut WorthQueryRetainedCapabilityAuthorization {
        &mut self.authorization
    }

    pub fn installed_capability_identity(&self) -> [u8; 32] {
        self.authorization.installed_capability_identity()
    }
}

pub type SortedInternalFieldRules = Vec<((String, String, String), Vec<AdmittedDisclosureRule>)>;
pub type SortedInternalRelationRules = Vec<(
    (String, String, String, RelationDirection),
    Vec<AdmittedDisclosureRule>,
)>;

pub struct WorthQueryApplicationInternalComputationAuthority {
    binding: WorthQueryApplicationGovernanceBinding,
    internal_field_rules: SortedInternalFieldRules,
    internal_relation_rules: SortedInternalRelationRules,
    disclosure_value: AspectValue,
}

#[derive(Clone, Debug)]
pub struct WorthQueryApplicationDisclosureDecision {
    classification: String,
    disclosed: BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
    omitted: BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
}

pub enum WorthQueryApplicationQueryGovernance {
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
pub enum WorthQueryApplicationQueryGovernanceDenialKind {
    Required,
    CapabilityMismatch,
    InternalComputationDenied,
}

pub fn admit_application_query_governance(
    contract: WorthQueryAdmittedApplicationDisclosureContract,
    pending: Option<WorthQueryPendingApplicationQueryGovernance>,
    binding: WorthQueryApplicationGovernanceBinding,
) -> Result<WorthQueryApplicationQueryGovernance, WorthQueryApplicationQueryGovernanceDenialKind> {
    preparation::prepare(contract, pending).map(|prepared| prepared.bind(binding))
}

pub mod binding;
pub use binding::WorthQueryApplicationGovernanceBinding;

pub use diagnostic::DiagnosticComparisonDenial;

pub use diagnostic_preparation::DiagnosticPreparationDenial;
