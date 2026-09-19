use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationOneShotDenialKind as ExecutionKind,
    WorthQueryApplicationQueryAdmissionDenialKind as AdmissionKind,
    WorthQueryEntityResolutionDenialKind as ScopeKind,
    WorthQueryOperationAuthorizationDenialKind as AuthorizationKind,
    WorthQueryPrincipalResolutionDenialKind as PrincipalKind,
    WorthQueryProductBranchAdmissionDenial as ProductKind,
};

use crate::application_entry::WorthQueryApplicationRequestQueryDenial;

use super::{output_posture, WorthQueryRequiredOutputRecoveryPosture};

pub(super) fn query_posture(
    denial: &WorthQueryApplicationRequestQueryDenial,
) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryApplicationRequestQueryDenial as Denial;
    match denial {
        Denial::BindingInstallation(_) | Denial::Limit(_) => {
            WorthQueryRequiredOutputRecoveryPosture::Terminal
        }
        Denial::ProductSelection(denial) => product_posture(*denial),
        Denial::OutputSettlement(output) => output_posture(output),
        Denial::PrincipalResolution(denial) => principal_posture(denial.kind()),
        Denial::ScopeResolution(denial) => scope_posture(denial.kind()),
        Denial::Admission(denial) => admission_posture(denial.kind()),
        Denial::Execution(denial) => execution_posture(denial.kind()),
        Denial::ContinuationExecution(_)
        | Denial::RequestMode
        | Denial::CapabilityInstallation(_)
        | Denial::CapabilityAdmission(_) => WorthQueryRequiredOutputRecoveryPosture::Terminal,
    }
}

fn product_posture(kind: ProductKind) -> WorthQueryRequiredOutputRecoveryPosture {
    if kind.is_transient() {
        WorthQueryRequiredOutputRecoveryPosture::Retryable
    } else {
        WorthQueryRequiredOutputRecoveryPosture::Terminal
    }
}

fn authorization_posture(kind: AuthorizationKind) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryRequiredOutputRecoveryPosture::{Retryable, Terminal};
    match kind {
        AuthorizationKind::ProductSecurityBasis(product) => product_posture(product),
        AuthorizationKind::Cancelled
        | AuthorizationKind::DeadlineExceeded
        | AuthorizationKind::TrustedTimeUnavailable
        | AuthorizationKind::ActiveSnapshotCapacityExhausted { .. }
        | AuthorizationKind::RetentionCapacityExhausted
        | AuthorizationKind::GraphWorkAdmissionUnavailable => Retryable,
        _ => Terminal,
    }
}

fn principal_posture(kind: PrincipalKind) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryRequiredOutputRecoveryPosture::{Retryable, Terminal};
    match kind {
        PrincipalKind::ExpiredAuthentication
        | PrincipalKind::Cancelled
        | PrincipalKind::DeadlineExceeded
        | PrincipalKind::BranchMaterializationSuspended
        | PrincipalKind::IdentityIndexUnavailable
        | PrincipalKind::ActiveSnapshotCapacityExhausted { .. }
        | PrincipalKind::RetentionCapacityExhausted => Retryable,
        PrincipalKind::PrimaryGraphNotInstalled
        | PrincipalKind::BindingNotInstalled
        | PrincipalKind::ForeignRuntime
        | PrincipalKind::StaleInstalledSchema
        | PrincipalKind::CorruptIdentityIndex
        | PrincipalKind::UnknownPrincipal
        | PrincipalKind::DisabledPrincipal
        | PrincipalKind::AmbiguousPrincipal
        | PrincipalKind::MissingPrincipalTarget
        | PrincipalKind::AmbiguousPrincipalTarget
        | PrincipalKind::WrongPrincipalTargetKind
        | PrincipalKind::StalePrincipalProof
        | PrincipalKind::SnapshotIdentityExhausted
        | PrincipalKind::RetentionIdentityExhausted => Terminal,
        _ => Terminal,
    }
}

fn scope_posture(kind: ScopeKind) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryRequiredOutputRecoveryPosture::{Retryable, Terminal};
    match kind {
        ScopeKind::Cancelled
        | ScopeKind::DeadlineExceeded
        | ScopeKind::EqualityIndexUnavailable
        | ScopeKind::ActiveSnapshotCapacityExhausted { .. }
        | ScopeKind::RetentionCapacityExhausted => Retryable,
        ScopeKind::PrimaryGraphNotInstalled
        | ScopeKind::FieldNotInstalled
        | ScopeKind::ValueEncodingRejected
        | ScopeKind::UnknownEntity
        | ScopeKind::AmbiguousEntity
        | ScopeKind::CorruptIdentityIndex
        | ScopeKind::ProjectionWorkBudgetExceeded
        | ScopeKind::SnapshotIdentityExhausted
        | ScopeKind::RetentionIdentityExhausted
        | ScopeKind::ForeignResolutionTruth => Terminal,
    }
}

fn admission_posture(kind: AdmissionKind) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryRequiredOutputRecoveryPosture::{Retryable, Terminal};
    match kind {
        AdmissionKind::Cancelled
        | AdmissionKind::DeadlineExceeded
        | AdmissionKind::BasisUnavailable
        | AdmissionKind::BranchMaterializationSuspended
        | AdmissionKind::ActiveSnapshotCapacityExhausted { .. }
        | AdmissionKind::RetentionCapacityExhausted
        | AdmissionKind::GraphWorkAdmissionUnavailable => Retryable,
        AdmissionKind::Authorization(kind) => authorization_posture(kind),
        AdmissionKind::InstalledQuery(_)
        | AdmissionKind::ForeignPrincipal
        | AdmissionKind::ForeignScope
        | AdmissionKind::StalePrincipal
        | AdmissionKind::StaleScope
        | AdmissionKind::ScopeTypeMismatch
        | AdmissionKind::BasisUnsupported
        | AdmissionKind::ForeignBasis
        | AdmissionKind::StaleBasis
        | AdmissionKind::WrongProviderBasis
        | AdmissionKind::ExpiredBasis
        | AdmissionKind::ForeignHistoricalReceipt
        | AdmissionKind::RuntimeSupportUnavailable
        | AdmissionKind::SnapshotIdentityExhausted
        | AdmissionKind::RetentionIdentityExhausted
        | AdmissionKind::ForeignContinuation
        | AdmissionKind::StaleContinuation
        | AdmissionKind::ContinuationParameterMismatch
        | AdmissionKind::ContinuationScopeMismatch
        | AdmissionKind::ContinuationProviderMismatch
        | AdmissionKind::ContinuationPageWidthUnsupported
        | AdmissionKind::LaneUnsupported
        | AdmissionKind::DisclosureGovernanceRequired
        | AdmissionKind::DisclosureContractInvalid
        | AdmissionKind::DisclosureAuthorizationMismatch
        | AdmissionKind::InternalComputationDenied
        | AdmissionKind::Parameter(_)
        | AdmissionKind::WorkLimitExceeded
        | AdmissionKind::CanonicalWorkDenied
        | AdmissionKind::GraphReadPlan(_)
        | AdmissionKind::ExecutionShapeUnsupported => Terminal,
        _ => Terminal,
    }
}

fn execution_posture(kind: ExecutionKind) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryRequiredOutputRecoveryPosture::{Retryable, Terminal};
    match kind {
        ExecutionKind::Cancelled
        | ExecutionKind::DeadlineExceeded
        | ExecutionKind::BasisUnavailable
        | ExecutionKind::ActiveSnapshotCapacityExhausted { .. }
        | ExecutionKind::RetentionCapacityExhausted
        | ExecutionKind::PredicateIndexUnavailable
        | ExecutionKind::TraversalUnavailable => Retryable,
        ExecutionKind::Authorization(kind) => authorization_posture(kind),
        ExecutionKind::ForeignPlan
        | ExecutionKind::StaleInstalledQuery
        | ExecutionKind::StalePrincipal
        | ExecutionKind::StaleScope
        | ExecutionKind::RetentionIdentityExhausted
        | ExecutionKind::SnapshotIdentityExhausted
        | ExecutionKind::ExpiredBasis
        | ExecutionKind::BasisReleaseFailed
        | ExecutionKind::PredicateLookupOverflow
        | ExecutionKind::ResultLimitExceeded
        | ExecutionKind::CardinalityMismatch
        | ExecutionKind::ProjectionUnavailable
        | ExecutionKind::Projection(_)
        | ExecutionKind::ResultBufferLimitExceeded
        | ExecutionKind::WorkLimitExceeded => Terminal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_world_capacity_is_retryable_but_foreign_authority_is_terminal() {
        assert_eq!(
            admission_posture(AdmissionKind::Authorization(
                AuthorizationKind::ProductSecurityBasis(ProductKind::ObservationCapacityExhausted,),
            )),
            WorthQueryRequiredOutputRecoveryPosture::Retryable,
        );
        assert_eq!(
            execution_posture(ExecutionKind::Authorization(
                AuthorizationKind::ForeignRuntime,
            )),
            WorthQueryRequiredOutputRecoveryPosture::Terminal,
        );
    }
}
