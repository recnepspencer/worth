//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryWorkflowApprovalAuthorityBasis {
    #[doc(hidden)]
    pub fn from_retained_approval_for_test(
        request: WorthQueryRetainedCapabilityRequest,
        principal: DecodedApprovalPrincipal,
        grant: EntityId,
        capability_authority_identity: String,
        lineage: DecodedApprovalLineage,
        support: Option<WorthQueryWorkflowApprovalSupportBasis>,
        dependencies: super::super::WorthQueryWorkflowApprovalDependencies,
        timeline: worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
        expiry: u64,
    ) -> Self {
        Self::from_retained_approval(
            request,
            principal,
            grant,
            capability_authority_identity,
            lineage,
            support,
            dependencies,
            timeline,
            expiry,
        )
    }
}
