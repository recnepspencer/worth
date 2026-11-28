//! HTTP disposition of the actual producer observation read.
use super::*;
use crate::http::server::relational_execution_denial;
use worth_query_host::facade::{
    application_contribution::WorthQueryProducerObservationReadDenial,
    primary_graph::{
        RelationalBranchBasisDenial, RelationalPredecessorBasisDenial, WorthQueryInvariantProjectionDenial,
        WorthQueryInvariantProjectionDenialKind,
    },
};

pub(super) fn denial(cause: WorthQueryProducerObservationReadDenial) -> BankHttpDenial {
    use WorthQueryProducerObservationReadDenial as D;
    match cause {
        D::PredecessorBasisUnavailable => unavailable(),
        D::PredecessorMismatch => stale(),
        D::Authorization(cause) => authorization_denial(cause.kind().into()),
        D::Control(cause) => source_observation::denial(*cause),
        D::Installation(_) => unavailable(),
        D::Projection(cause) => projection(cause),
        D::Decision(_) => malformed(),
        D::UndeclaredObservationRead => operator(),
        D::Native(cause) => native(cause),
    }
}

fn native(cause: RelationalPredecessorBasisDenial) -> BankHttpDenial {
    use RelationalPredecessorBasisDenial as D;
    match cause {
        D::PredecessorBasisUnavailable | D::PredecessorBasisUnavailableForOrigin(_) => {
            unavailable()
        }
        D::PredecessorMismatch => stale(),
        D::BasisAdmission(cause) => native_basis(cause),
        D::Selection(cause) => publication_read::denial(cause),
        D::Execution(cause) => relational_execution_denial::denial(cause),
    }
}

fn native_basis(cause: RelationalBranchBasisDenial) -> BankHttpDenial {
    use RelationalBranchBasisDenial as D;
    match cause {
        D::Cancelled => cancelled(),
        D::TimedOut => deadline(),
        D::RetentionCapacityExhausted | D::OwnerUnavailable | D::MaterializationUnavailable => {
            unavailable()
        }
        D::RetentionIdentityExhausted
        | D::SnapshotIdentityExhausted
        | D::UnavailableRetainedTarget => {
            BankHttpDenial::new(Kind::Unavailable, Next::ContactOperator)
        }
        D::UnknownBranch(_)
        | D::ArchivedBranch(_)
        | D::DeletingBranch(_)
        | D::StaleReferenceGeneration
        | D::WrongBranchLocalTruthVersion => stale(),
        D::MalformedDescriptor
        | D::UnsupportedDescriptorVersion { .. }
        | D::ForeignRuntime { .. }
        | D::EmptyCommittedTargetMismatch
        | D::WrongImmutableTarget
        | D::MixedAxis(_)
        | D::OwnerFailure => operator(),
    }
}

fn projection(cause: WorthQueryInvariantProjectionDenial) -> BankHttpDenial {
    use WorthQueryInvariantProjectionDenialKind as D;
    match cause.kind() {
        D::ValueDecode(cause) => value_decode(cause),
        D::ExecutionRequest(cause) => relational_execution_denial::denial(cause),
        D::BasisUnavailable => unavailable(),
        D::ForeignPhase | D::ForeignBasis => operator(),
        D::SourceRetentionDenied => match cause.source_retention_denial() {
            Some(cause) => retained_source(cause),
            None => operator(),
        },
        D::ActiveSnapshotCapacityExhausted { .. } | D::RetentionCapacityExhausted => {
            BankHttpDenial::new(Kind::ResourceExhausted, Next::Retry)
        }
        D::SnapshotIdentityExhausted | D::RetentionIdentityExhausted => {
            BankHttpDenial::new(Kind::ResourceExhausted, Next::ContactOperator)
        }
        D::WorkBudgetExceeded => exhausted(),
    }
}

fn retained_source(cause: &worth_query_host::facade::primary_graph::StoreDenial) -> BankHttpDenial {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestInterruption as Stop;
    use worth_query_host::facade::primary_graph::StoreDenial as D;
    match cause {
        D::Allocation(cause) => managed_index::allocation(cause),
        D::ExecutionRequest(cause) => crate::http::server::advancement_denial::request(*cause),
        D::RequestInterruption(Stop::Cancelled) => cancelled(),
        D::RequestInterruption(Stop::DeadlineExceeded) => deadline(),
        D::Content(cause) => relational_execution_denial::content(*cause),
        D::PhysicalAllocation => unavailable(),
        D::Representability | D::ConflictingBody | D::ContentInconsistency | D::InvalidOrdinal => {
            operator()
        }
    }
}

fn operator() -> BankHttpDenial {
    BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator)
}

#[cfg(test)]
mod tests;
