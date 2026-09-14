use bank_server::{
    BankApplicationOneShotDenialKind, BankApplicationOutputSettlementDenialKind,
    BankApplicationProjectionDenialKind, BankApplicationQueryAdmissionDenialKind,
    BankApplicationQueryDenial, BankApplicationQueryParameterDenialKind,
    BankAuthorizationDenialKind, BankEntityResolutionDenialKind, BankGraphReadPlanReviewDenialKind,
};

use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};

pub(super) fn query_denial(denial: BankApplicationQueryDenial) -> BankHttpDenial {
    match denial {
        BankApplicationQueryDenial::Installation(_)
        | BankApplicationQueryDenial::CapabilityInstallation(_)
        | BankApplicationQueryDenial::PreviewSession(_)
        | BankApplicationQueryDenial::ContinuationExecution(_)
        | BankApplicationQueryDenial::LiveOpen(_) => unavailable(),
        BankApplicationQueryDenial::PrincipalResolution(_) => stale(),
        BankApplicationQueryDenial::Limit(_) => exhausted(),
        BankApplicationQueryDenial::ProductSelection(_) => stale(),
        BankApplicationQueryDenial::HistoricalCommitUnavailable => stale(),
        BankApplicationQueryDenial::CapabilityAdmission(denial) => authorization(denial.kind()),
        BankApplicationQueryDenial::ScopeResolution(denial) => entity(denial.kind()),
        BankApplicationQueryDenial::Admission(denial) => admission(denial.kind()),
        BankApplicationQueryDenial::Execution(denial) => execution(denial.kind()),
        BankApplicationQueryDenial::OutputSettlement(kind) => output_settlement(kind),
    }
}

fn admission(kind: BankApplicationQueryAdmissionDenialKind) -> BankHttpDenial {
    use BankApplicationQueryAdmissionDenialKind as Admission;
    match kind {
        Admission::Authorization(kind) => authorization(kind),
        Admission::Cancelled => cancelled(),
        Admission::DeadlineExceeded => deadline(),
        Admission::StalePrincipal
        | Admission::StaleScope
        | Admission::StaleBasis
        | Admission::ExpiredBasis
        | Admission::StaleContinuation => stale(),
        Admission::ForeignPrincipal
        | Admission::ForeignScope
        | Admission::ScopeTypeMismatch
        | Admission::ForeignBasis
        | Admission::WrongProviderBasis
        | Admission::ForeignHistoricalReceipt
        | Admission::ForeignContinuation
        | Admission::ContinuationParameterMismatch
        | Admission::ContinuationScopeMismatch
        | Admission::ContinuationProviderMismatch
        | Admission::DisclosureAuthorizationMismatch => permission_denied(),
        Admission::Parameter(kind) => parameter(kind),
        Admission::WorkLimitExceeded
        | Admission::CanonicalWorkDenied
        | Admission::GraphReadPlan(BankGraphReadPlanReviewDenialKind::BudgetExceeded) => {
            exhausted()
        }
        Admission::InstalledQuery(_)
        | Admission::BasisUnsupported
        | Admission::BasisUnavailable
        | Admission::RuntimeSupportUnavailable
        | Admission::ContinuationPageWidthUnsupported
        | Admission::LaneUnsupported
        | Admission::GraphReadPlan(_)
        | Admission::GraphWorkAdmissionUnavailable
        | Admission::ExecutionShapeUnsupported => unavailable(),
        Admission::ActiveSnapshotCapacityExhausted { .. }
        | Admission::SnapshotIdentityExhausted
        | Admission::RetentionCapacityExhausted
        | Admission::RetentionIdentityExhausted => exhausted(),
        Admission::DisclosureGovernanceRequired
        | Admission::DisclosureContractInvalid
        | Admission::InternalComputationDenied => internal_denied(),
    }
}

fn execution(kind: BankApplicationOneShotDenialKind) -> BankHttpDenial {
    use BankApplicationOneShotDenialKind as Execution;
    match kind {
        Execution::Authorization(kind) => authorization(kind),
        Execution::Cancelled => cancelled(),
        Execution::DeadlineExceeded => deadline(),
        Execution::StaleInstalledQuery | Execution::StalePrincipal | Execution::StaleScope => {
            stale()
        }
        Execution::ForeignPlan => permission_denied(),
        Execution::ResultLimitExceeded
        | Execution::ResultBufferLimitExceeded
        | Execution::WorkLimitExceeded
        | Execution::PredicateLookupOverflow
        | Execution::ActiveSnapshotCapacityExhausted { .. }
        | Execution::RetentionCapacityExhausted
        | Execution::RetentionIdentityExhausted
        | Execution::SnapshotIdentityExhausted => exhausted(),
        Execution::Projection(kind) => projection(kind),
        Execution::BasisUnavailable
        | Execution::ExpiredBasis
        | Execution::BasisReleaseFailed
        | Execution::PredicateIndexUnavailable
        | Execution::TraversalUnavailable
        | Execution::ProjectionUnavailable => unavailable(),
        Execution::CardinalityMismatch => internal_denied(),
    }
}

fn parameter(kind: BankApplicationQueryParameterDenialKind) -> BankHttpDenial {
    use BankApplicationQueryParameterDenialKind as Parameter;
    match kind {
        Parameter::ParameterSetMismatch | Parameter::ParameterTypeMismatch => malformed(),
        Parameter::CanonicalEntryBudgetExceeded | Parameter::CanonicalEncodedByteBudgetExceeded => {
            exhausted()
        }
        Parameter::CanonicalDigestSlotRejected => internal_denied(),
    }
}

fn projection(kind: BankApplicationProjectionDenialKind) -> BankHttpDenial {
    use BankApplicationProjectionDenialKind as Projection;
    match kind {
        Projection::DomainProjectionRejected => permission_denied(),
        Projection::FieldNotProjected
        | Projection::FieldContractMismatch
        | Projection::FieldTypeMismatch
        | Projection::FieldOmitted
        | Projection::RelationNotProjected
        | Projection::RelationContractMismatch
        | Projection::RelationCardinalityMismatch
        | Projection::RelationOmitted => internal_denied(),
    }
}

fn entity(kind: BankEntityResolutionDenialKind) -> BankHttpDenial {
    use BankEntityResolutionDenialKind as Entity;
    match kind {
        Entity::Cancelled => cancelled(),
        Entity::DeadlineExceeded => deadline(),
        Entity::UnknownEntity => BankHttpDenial::new(Kind::NotFound, Next::CorrectRequest),
        Entity::ValueEncodingRejected => malformed(),
        Entity::ProjectionWorkBudgetExceeded
        | Entity::ActiveSnapshotCapacityExhausted { .. }
        | Entity::SnapshotIdentityExhausted
        | Entity::RetentionCapacityExhausted
        | Entity::RetentionIdentityExhausted => exhausted(),
        Entity::ForeignResolutionTruth => stale(),
        Entity::PrimaryGraphNotInstalled
        | Entity::FieldNotInstalled
        | Entity::EqualityIndexUnavailable => unavailable(),
        Entity::AmbiguousEntity | Entity::CorruptIdentityIndex => internal_denied(),
    }
}

fn authorization(kind: BankAuthorizationDenialKind) -> BankHttpDenial {
    use BankAuthorizationDenialKind as Authorization;
    match kind {
        Authorization::Cancelled => cancelled(),
        Authorization::DeadlineExceeded => deadline(),
        Authorization::ExpiredAuthentication => {
            BankHttpDenial::new(Kind::Unauthenticated, Next::Authenticate)
        }
        Authorization::StaleInstalledSchema
        | Authorization::StaleInstalledOperation
        | Authorization::StalePrincipal
        | Authorization::StaleScope
        | Authorization::StaleAuthorization
        | Authorization::DelegationLineageChanged => stale(),
        Authorization::CanonicalWorkDenied
        | Authorization::GrantSelectionLimitExceeded
        | Authorization::ActiveSnapshotCapacityExhausted { .. }
        | Authorization::SnapshotIdentityExhausted
        | Authorization::RetentionCapacityExhausted
        | Authorization::RetentionIdentityExhausted => exhausted(),
        Authorization::InvalidOperationInput => malformed(),
        Authorization::ForeignRuntime
        | Authorization::MutationPreconditionRejected
        | Authorization::CapabilityGrantMissing
        | Authorization::CapabilityAuthorizationMissing
        | Authorization::PurposeMismatch
        | Authorization::ExplicitDenyRuleMatched
        | Authorization::ConflictRuleMatched
        | Authorization::SeparationOfDutyRuleMatched
        | Authorization::DistinctActorRuleMatched
        | Authorization::CapabilityRequired
        | Authorization::CapabilityNotRequired
        | Authorization::CapabilityExpired
        | Authorization::ElevationRequired
        | Authorization::ElevationNotApplicable
        | Authorization::ElevationExpired
        | Authorization::ElevationInactive
        | Authorization::ElevationSelfApproval
        | Authorization::ElevationApproverConflict
        | Authorization::ElevationTransitionRequired
        | Authorization::ElevationLifecycleRoleMismatch
        | Authorization::ElevationRequestRejected
        | Authorization::ElevationApprovalRejected
        | Authorization::ElevationCloseRejected
        | Authorization::MandatoryReviewRejected
        | Authorization::ElevationDurationExceeded
        | Authorization::DelegationRejected
        | Authorization::DelegationTransitionRequired
        | Authorization::DelegationDepthExceeded
        | Authorization::DelegationCycle
        | Authorization::ScopeMismatch
        | Authorization::ProductSecurityBasis(_)
        | Authorization::PermissionDenied => permission_denied(),
        Authorization::TrustedTimeUnavailable
        | Authorization::GraphWorkAdmissionUnavailable
        | Authorization::PolicyNotInstalled => unavailable(),
        Authorization::CapabilityProjectionRejected
        | Authorization::ElevationProjectionRejected
        | Authorization::AdmissionIdentityExhausted
        | Authorization::InvalidInstalledPolicy
        | Authorization::RelationalObservationRejected
        | Authorization::BridgeEvaluationRejected
        | Authorization::InconsistentDecision => internal_denied(),
    }
}

fn output_settlement(kind: BankApplicationOutputSettlementDenialKind) -> BankHttpDenial {
    use BankApplicationOutputSettlementDenialKind as Settlement;
    match kind {
        Settlement::Cancelled => cancelled(),
        Settlement::TimedOut => deadline(),
        Settlement::Superseded
        | Settlement::ForeignSource
        | Settlement::ForeignDemand
        | Settlement::ForeignSettlement
        | Settlement::RetainedBasisUnavailable
        | Settlement::Closed => stale(),
        Settlement::WorkBudgetExceeded | Settlement::RetentionBudgetExceeded => exhausted(),
        Settlement::MissingApplicableProducer
        | Settlement::AmbiguousApplicableProducer
        | Settlement::ProducerUnavailable
        | Settlement::SchedulingRejected => unavailable(),
    }
}

const fn malformed() -> BankHttpDenial {
    BankHttpDenial::new(Kind::MalformedRequest, Next::CorrectRequest)
}

const fn permission_denied() -> BankHttpDenial {
    BankHttpDenial::new(Kind::PermissionDenied, Next::None)
}

const fn cancelled() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Cancelled, Next::Retry)
}

const fn deadline() -> BankHttpDenial {
    BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry)
}

const fn stale() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Stale, Next::Refresh)
}

const fn unavailable() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Unavailable, Next::Retry)
}

const fn exhausted() -> BankHttpDenial {
    BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest)
}

const fn internal_denied() -> BankHttpDenial {
    BankHttpDenial::new(Kind::InternalDenied, Next::None)
}
