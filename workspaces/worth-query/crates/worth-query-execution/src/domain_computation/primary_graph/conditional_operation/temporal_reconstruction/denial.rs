use super::super::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;

pub(in crate::domain_computation::primary_graph::conditional_operation) fn bridge_reconstruction_denial(
    detail: impl Into<String>,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    reconstruction_denial(
        WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionIntent,
        detail,
    )
}

pub(in crate::domain_computation::primary_graph::conditional_operation) fn snapshot_capacity_reconstruction_denial(
    maximum_active_snapshots: usize,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    reconstruction_denial(
        WorthQueryConditionalRuntimeInstallationDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        "conditional reconstruction deferred by the relational snapshot capacity owner",
    )
}

pub(in crate::domain_computation::primary_graph::conditional_operation) fn retention_capacity_reconstruction_denial(
) -> WorthQueryConditionalRuntimeInstallationDenial {
    reconstruction_denial(
        WorthQueryConditionalRuntimeInstallationDenialKind::RetentionCapacityExhausted,
        "conditional reconstruction deferred by the relational retention owner",
    )
}

pub(in crate::domain_computation::primary_graph::conditional_operation) fn retention_identity_reconstruction_denial(
) -> WorthQueryConditionalRuntimeInstallationDenial {
    reconstruction_denial(
        WorthQueryConditionalRuntimeInstallationDenialKind::RetentionIdentityExhausted,
        "conditional reconstruction exhausted relational retention identity space",
    )
}

pub(in crate::domain_computation::primary_graph::conditional_operation) fn snapshot_identity_reconstruction_denial(
) -> WorthQueryConditionalRuntimeInstallationDenial {
    reconstruction_denial(
        WorthQueryConditionalRuntimeInstallationDenialKind::SnapshotIdentityExhausted,
        "conditional reconstruction exhausted relational snapshot identity space",
    )
}

pub(in crate::domain_computation::primary_graph::conditional_operation) fn reconstruction_denial(
    kind: WorthQueryConditionalRuntimeInstallationDenialKind,
    detail: impl Into<String>,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::new(kind, detail)
}

pub(super) fn principal(
    denial: crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind as Kind;
    match denial.kind() {
        Kind::Stopped(stop) => read_stopped(stop),

        Kind::Refused(PrincipalRefusal::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        }) => snapshot_capacity_reconstruction_denial(maximum_active_snapshots),
        Kind::Refused(PrincipalRefusal::RetentionCapacityExhausted) => {
            retention_capacity_reconstruction_denial()
        }
        Kind::Refused(PrincipalRefusal::RetentionIdentityExhausted) => {
            retention_identity_reconstruction_denial()
        }
        Kind::Refused(PrincipalRefusal::SnapshotIdentityExhausted) => {
            snapshot_identity_reconstruction_denial()
        }
        Kind::Refused(PrincipalRefusal::ForeignPhase)
        | Kind::Refused(PrincipalRefusal::PrimaryGraphNotInstalled)
        | Kind::Refused(PrincipalRefusal::BindingNotInstalled)
        | Kind::Refused(PrincipalRefusal::ForeignRuntime)
        | Kind::Refused(PrincipalRefusal::StaleInstalledSchema)
        | Kind::Refused(PrincipalRefusal::ExpiredAuthentication)
        | Kind::Refused(PrincipalRefusal::BranchMaterializationSuspended)
        | Kind::Refused(PrincipalRefusal::IdentityIndexUnavailable)
        | Kind::Refused(PrincipalRefusal::CorruptIdentityIndex)
        | Kind::Refused(PrincipalRefusal::UnknownPrincipal)
        | Kind::Refused(PrincipalRefusal::DisabledPrincipal)
        | Kind::Refused(PrincipalRefusal::AmbiguousPrincipal)
        | Kind::Refused(PrincipalRefusal::MissingPrincipalTarget)
        | Kind::Refused(PrincipalRefusal::AmbiguousPrincipalTarget)
        | Kind::Refused(PrincipalRefusal::WrongPrincipalTargetKind)
        | Kind::Refused(PrincipalRefusal::StalePrincipalProof) => reconstruction_denial(
            WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionPrincipal,
            format!("{:?}: {}", denial.kind(), denial.binding()),
        ),
    }
}

pub(super) fn entity(
    denial: crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    use crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind as Kind;
    match denial.kind() {
        Kind::Refused(ResolutionRefusal::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        }) => snapshot_capacity_reconstruction_denial(maximum_active_snapshots),
        Kind::Refused(ResolutionRefusal::RetentionCapacityExhausted) => {
            retention_capacity_reconstruction_denial()
        }
        Kind::Refused(ResolutionRefusal::RetentionIdentityExhausted) => {
            retention_identity_reconstruction_denial()
        }
        Kind::Refused(ResolutionRefusal::SnapshotIdentityExhausted) => {
            snapshot_identity_reconstruction_denial()
        }
        Kind::Stopped(cause) => read_stopped(
            crate::facade::primary_graph::relational_execution_kind(cause, None),
        ),
        Kind::Refused(_) => reconstruction_denial(
            WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionScope,
            format!("{:?}: {}", denial.kind(), denial.subject()),
        ),
    }
}

pub(super) fn query_authorization(
    denial: super::super::WorthQueryTemporalQueryAuthorizationDenial,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    use super::super::WorthQueryTemporalQueryAuthorizationDenial as Denial;
    match denial {
        Denial::Query(query) => query::admission(query.kind(), query.to_string()),
        Denial::Authorization(authorization) => {
            query::authorization(authorization.kind(), authorization.to_string())
        }
    }
}
mod query;

pub(super) fn one_shot(
    denial: crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenial,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind as Kind;
    match denial.kind() {
        Kind::ExecutionRequest(kind) => read_stopped(Ok(kind)),
        Kind::Cancelled => read_stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
        )),
        Kind::DeadlineExceeded => read_stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
        )),

        Kind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => snapshot_capacity_reconstruction_denial(maximum_active_snapshots),
        Kind::RetentionCapacityExhausted => retention_capacity_reconstruction_denial(),
        Kind::RetentionIdentityExhausted => retention_identity_reconstruction_denial(),
        Kind::SnapshotIdentityExhausted => snapshot_identity_reconstruction_denial(),
        Kind::ForeignPlan
        | Kind::StaleInstalledQuery
        | Kind::StalePrincipal
        | Kind::StaleScope
        | Kind::Authorization(..)
        | Kind::BasisUnavailable
        | Kind::ExpiredBasis
        | Kind::BasisReleaseFailed
        | Kind::PredicateIndexUnavailable
        | Kind::ResultLimitExceeded
        | Kind::CardinalityMismatch
        | Kind::TraversalUnavailable
        | Kind::ProjectionUnavailable
        | Kind::ExpressionContextUnavailable
        | Kind::ExpressionContextIncomplete
        | Kind::Projection(..)
        | Kind::ResultBufferLimitExceeded
        | Kind::WorkLimitExceeded
        | Kind::SourceIdentityExhausted => reconstruction_denial(
            WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionQuery,
            format!("{:?}: {}", denial.kind(), denial.subject()),
        ),
    }
}

pub(in crate::domain_computation::primary_graph::conditional_operation) fn read_stopped(
    stop: Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    reconstruction_denial(
        WorthQueryConditionalRuntimeInstallationDenialKind::ReadExecution(stop),
        "conditional reconstruction read stopped",
    )
}
