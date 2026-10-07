use crate::runtime::{
    WorthQueryReadDenialKind, WorthQueryRuntimeError, WorthQueryRuntimeMissingComponent,
    WorthQueryStopClass,
};

use super::super::WorthQueryReadContextDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryReadNextAction {
    ReviseDeclaration,
    SupplyFreshBasis,
    SupplyPolicyAuthority,
    SupplyTenantAuthority,
    SupplyBranchAuthority,
    SupplyRelationshipProofAuthority,
    ConfigureRuntime(WorthQueryRuntimeMissingComponent),
    SelectSupportedCapability,
    InspectOperationalFailure,
}

pub(super) fn classify_runtime_next_action(
    error: &WorthQueryRuntimeError,
) -> WorthQueryReadNextAction {
    match error.stop_class() {
        WorthQueryStopClass::ReadCompositionDenied { denial } => {
            next_action_for_read_denial(denial.kind())
        }
        WorthQueryStopClass::MissingRuntimeComponent { component } => {
            WorthQueryReadNextAction::ConfigureRuntime(component)
        }
        WorthQueryStopClass::SharedReadStaleBasis { .. } => {
            WorthQueryReadNextAction::SupplyFreshBasis
        }
        WorthQueryStopClass::FamilyAdmissionDenied { .. } => {
            WorthQueryReadNextAction::SelectSupportedCapability
        }
        _ => WorthQueryReadNextAction::InspectOperationalFailure,
    }
}

pub(super) fn classify_context_next_action(
    denial: &WorthQueryReadContextDenial,
) -> WorthQueryReadNextAction {
    use super::super::WorthQueryReadContextDenialSource;
    use crate::policy_basis::PolicyTenantAdmissionFailureClass as PolicyFailure;
    use crate::policy_narrowing::PolicyNarrowingFailureClass as NarrowingFailure;
    use crate::relationship_proof::RelationshipProofFailureClass as RelationshipFailure;
    use crate::runtime::WorthQueryGraphReadAccessAuthorityDenialKind as AuthorityFailure;

    match denial.source() {
        WorthQueryReadContextDenialSource::MissingRelationshipProof => {
            WorthQueryReadNextAction::SupplyRelationshipProofAuthority
        }
        WorthQueryReadContextDenialSource::PolicyTenant(error) => match error.failure_class() {
            PolicyFailure::BranchAccessDenied => WorthQueryReadNextAction::SupplyBranchAuthority,
            PolicyFailure::BasisMismatch => WorthQueryReadNextAction::SupplyFreshBasis,
            PolicyFailure::CrossTenant | PolicyFailure::TenantAdmissionDenied => {
                WorthQueryReadNextAction::SupplyTenantAuthority
            }
            PolicyFailure::UnsupportedExecutionMode => {
                WorthQueryReadNextAction::SelectSupportedCapability
            }
            PolicyFailure::PolicyQueryFamilyDenied
            | PolicyFailure::RawMiddlewarePolicySourceForbidden
            | PolicyFailure::StalePolicyAuthority
            | PolicyFailure::PolicyWorkBudgetDenied
            | PolicyFailure::SavedQueryPolicyTenantBypassForbidden => {
                WorthQueryReadNextAction::SupplyPolicyAuthority
            }
        },
        WorthQueryReadContextDenialSource::PolicyNarrowing(error) => match error.failure_class() {
            NarrowingFailure::RelationshipProofDenied(_) => {
                WorthQueryReadNextAction::SupplyRelationshipProofAuthority
            }
            NarrowingFailure::CanonicalQueryDigestMismatch
            | NarrowingFailure::PolicyMaskAuthorityMismatch
            | NarrowingFailure::AuthorizedProjectionDenied(_)
            | NarrowingFailure::UnknownNarrowingCost
            | NarrowingFailure::UnboundedDerivedInfluence
            | NarrowingFailure::UnboundedProofTopology
            | NarrowingFailure::DigestPartBudgetExceeded => {
                WorthQueryReadNextAction::SupplyPolicyAuthority
            }
        },
        WorthQueryReadContextDenialSource::RelationshipProof(error) => {
            match error.failure_class() {
                RelationshipFailure::QueryShapeMismatch
                | RelationshipFailure::PolicyMismatch
                | RelationshipFailure::TenantSchemaMismatch
                | RelationshipFailure::MissingProofBasis
                | RelationshipFailure::HostCallbackForbidden
                | RelationshipFailure::UnboundedRecursiveWalk
                | RelationshipFailure::RelationshipProofBudgetExceeded
                | RelationshipFailure::UnboundedProofTopology => {
                    WorthQueryReadNextAction::SupplyRelationshipProofAuthority
                }
            }
        }
        WorthQueryReadContextDenialSource::GraphAuthority(error) => match error.kind() {
            AuthorityFailure::PolicyTenantDenied => WorthQueryReadNextAction::SupplyPolicyAuthority,
            AuthorityFailure::PolicyTenantBasisScopeMismatch => {
                WorthQueryReadNextAction::SupplyFreshBasis
            }
            AuthorityFailure::RelationshipProofRequiresPolicyTenantContext
            | AuthorityFailure::RelationshipProofPolicyTenantMismatch => {
                WorthQueryReadNextAction::SupplyRelationshipProofAuthority
            }
        },
    }
}

fn next_action_for_read_denial(kind: &WorthQueryReadDenialKind) -> WorthQueryReadNextAction {
    match kind {
        WorthQueryReadDenialKind::Handle(_) => WorthQueryReadNextAction::InspectOperationalFailure,
        WorthQueryReadDenialKind::BasisResolutionDenied
        | WorthQueryReadDenialKind::BasisPreflightDenied => {
            WorthQueryReadNextAction::SupplyFreshBasis
        }
        WorthQueryReadDenialKind::InvalidRoot
        | WorthQueryReadDenialKind::BuiltInOperatorDenied
        | WorthQueryReadDenialKind::RelationshipProofAdmissionDenied
        | WorthQueryReadDenialKind::ScopeShapeDenied
        | WorthQueryReadDenialKind::InstalledOperationBindingDenied
        | WorthQueryReadDenialKind::AuthoringDenied
        | WorthQueryReadDenialKind::CanonicalizationDenied
        | WorthQueryReadDenialKind::ValidationDenied
        | WorthQueryReadDenialKind::PlanningDenied => WorthQueryReadNextAction::ReviseDeclaration,
        WorthQueryReadDenialKind::ExecutionDenied => {
            WorthQueryReadNextAction::InspectOperationalFailure
        }
    }
}

pub(super) fn classify_planning_next_action(
    denial: &crate::runtime::WorthQueryReadDenial,
) -> WorthQueryReadNextAction {
    next_action_for_read_denial(denial.kind())
}
