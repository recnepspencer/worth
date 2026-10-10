use bank_server::{
    BankApplicationOneShotDenialKind, BankApplicationOutputSettlementDenialKind,
    BankApplicationProjectionDenialKind, BankApplicationQueryAdmissionDenialKind,
    BankApplicationQueryDenial, BankApplicationQueryParameterDenialKind,
    BankEntityResolutionDenialKind, BankGraphReadPlanReviewDenialKind,
    BankProductSelectionDenialKind,
};

use super::authorization_denial::authorization_denial;

use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};

pub(super) fn query_denial(denial: BankApplicationQueryDenial) -> BankHttpDenial {
    match denial {
        BankApplicationQueryDenial::ExecutionRequest(cause) => {
            super::advancement_denial::advancement(cause)
        }
        BankApplicationQueryDenial::RequestMode
        | BankApplicationQueryDenial::LiveRetainedBasis
        | BankApplicationQueryDenial::LiveControls(_) => malformed(),
        BankApplicationQueryDenial::Installation(_)
        | BankApplicationQueryDenial::CapabilityInstallation(_)
        | BankApplicationQueryDenial::PreviewSession(_)
        | BankApplicationQueryDenial::ContinuationExecution(_)
        | BankApplicationQueryDenial::LiveOpen(_) => unavailable(),
        BankApplicationQueryDenial::PrincipalResolution(_) => stale(),
        BankApplicationQueryDenial::Limit(_) => exhausted(),
        BankApplicationQueryDenial::ProductSelection(kind) => product_selection(kind),
        BankApplicationQueryDenial::HistoricalCommitUnavailable => stale(),
        BankApplicationQueryDenial::CapabilityAdmission(denial) => {
            authorization_denial(denial.kind())
        }
        BankApplicationQueryDenial::ScopeResolution(denial) => entity(denial.kind()),
        BankApplicationQueryDenial::Admission(denial) => admission(denial.kind()),
        BankApplicationQueryDenial::Execution(denial) => execution(denial.kind()),
        BankApplicationQueryDenial::OutputSettlement(kind) => output_settlement(kind),
    }
}

fn product_selection(kind: BankProductSelectionDenialKind) -> BankHttpDenial {
    use BankProductSelectionDenialKind as Selection;
    match kind {
        Selection::ForeignOwner
        | Selection::RetiredBranch
        | Selection::IncarnationChanged
        | Selection::ObservationRejected
        | Selection::ObservationStaleSourceHead => stale(),
        Selection::ObservationCancelled => cancelled(),
        Selection::ObservationDeadlineExceeded => deadline(),
        Selection::ObservationStatePoisoned => internal_denied(),
        Selection::OwnerUnavailable
        | Selection::ProductActivationUnavailable
        | Selection::RelationalBasisUnavailable
        | Selection::RelationalSnapshotUnavailable
        | Selection::BridgeSourceUnavailable => unavailable(),
        Selection::ObservationCapacityExhausted
        | Selection::CustodyCapacityExhausted
        | Selection::ActiveSnapshotCapacityExhausted { .. }
        | Selection::RetentionCapacityExhausted => {
            BankHttpDenial::new(Kind::ResourceExhausted, Next::Retry)
        }
        Selection::ObservationIdentityExhausted
        | Selection::ObservationAccountingOverflow
        | Selection::RetentionIdentityExhausted
        | Selection::SnapshotIdentityExhausted => {
            BankHttpDenial::new(Kind::ResourceExhausted, Next::ContactOperator)
        }
    }
}

fn admission(kind: BankApplicationQueryAdmissionDenialKind) -> BankHttpDenial {
    use BankApplicationQueryAdmissionDenialKind as Admission;
    match kind {
        Admission::Authorization(kind) => authorization_denial(kind),
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
        Execution::Authorization(kind) => authorization_denial(kind),
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
        | Execution::SnapshotIdentityExhausted
        | Execution::SourceIdentityExhausted => exhausted(),
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
        | Entity::ProjectionPreparationMemoryExhausted
        | Entity::ActiveSnapshotCapacityExhausted { .. }
        | Entity::SnapshotIdentityExhausted
        | Entity::RetentionCapacityExhausted
        | Entity::RetentionIdentityExhausted => exhausted(),
        Entity::ForeignResolutionTruth => stale(),
        Entity::PrimaryGraphNotInstalled
        | Entity::FieldNotInstalled
        | Entity::EqualityIndexUnavailable => unavailable(),
        // The server's own handler sets a selection's candidate limit, so no
        // retry and no client correction can pass either refusal.
        Entity::AmbiguousEntity
        | Entity::CorruptIdentityIndex
        | Entity::InvalidCandidateLimit
        | Entity::CandidateLimitExceeded { .. } => internal_denied(),
    }
}

fn output_settlement(kind: BankApplicationOutputSettlementDenialKind) -> BankHttpDenial {
    use BankApplicationOutputSettlementDenialKind as Settlement;
    match kind {
        Settlement::BridgeConditional(_)
        | Settlement::CorrespondenceDelivery(_)
        | Settlement::ProductDelivery(_) => unavailable(),
        Settlement::ExecutionRequest(cause) => super::advancement_denial::advancement(cause),
        Settlement::SourceQueryInstallation(_) => unavailable(),
        Settlement::SourcePrincipal(_) => stale(),
        Settlement::SourceScope(kind) => entity(kind),
        Settlement::SourceQueryAdmission(kind) => admission(kind),
        Settlement::SourceQueryExecution(kind) => execution(kind),
        Settlement::RequestAuthorization(kind) => authorization_denial(kind),
        Settlement::Cancelled => cancelled(),
        Settlement::TimedOut => deadline(),
        Settlement::ProducerDomainDenied => malformed(),
        Settlement::Superseded
        | Settlement::PublicationStale
        | Settlement::ForeignSource
        | Settlement::ForeignDemand
        | Settlement::ForeignSettlement
        | Settlement::RetainedBasisUnavailable
        | Settlement::Closed => stale(),
        Settlement::WorkBudgetExceeded
        | Settlement::RetentionBudgetExceeded
        | Settlement::PublicationCapacityExceeded => exhausted(),
        Settlement::MissingApplicableProducer
        | Settlement::AmbiguousApplicableProducer
        | Settlement::ProducerUnavailable
        | Settlement::SchedulingRejected
        | Settlement::SchedulingDeferred
        | Settlement::NoEffect => unavailable(),
        Settlement::ProductSelection(kind) => product_selection(kind),
        Settlement::DuplicatePerformedSource | Settlement::IncompleteDependencyCoverage => {
            internal_denied()
        }
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
