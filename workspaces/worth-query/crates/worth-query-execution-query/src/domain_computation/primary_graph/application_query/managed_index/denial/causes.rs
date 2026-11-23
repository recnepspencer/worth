//! Closed descriptive cause projection; private proof/session types stay private.
use super::super::{
    execution::ProviderExecutionDenial, retention::WorthQueryManagedIndexLifecycleDenial,
    selection::SelectionDenial, source_acquisition::AcquisitionCause,
    source_completion::SourceDenial,
};
use crate::domain_computation::{
    primary_graph::application_query,
    provider_session::{
        WorthQueryManagedGraphReadDenial, WorthQueryManagedGraphWorkSessionStartDenial,
        WorthQueryManagedIndexSessionDenial,
    },
};
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkCapacityAdmissionStop;

pub use worth_query_execution_lineage::facade::primary_graph::application_query::managed_index::denial::causes::{
    WorthQueryManagedIndexDenialCause, WorthQueryManagedIndexRefusal,
};
use WorthQueryManagedIndexDenialCause as Cause;
use WorthQueryManagedIndexRefusal as Refusal;

pub fn provider(cause: &ProviderExecutionDenial) -> Cause<'_> {
    match cause {
        ProviderExecutionDenial::SourceVerification(cause) => Cause::SourceVerification {
            kind: cause.kind(),
            authorization: cause.request_authority().map(|denial| denial.kind()),
            execution: cause.relational_execution_cause(),
        },
        ProviderExecutionDenial::NativeSource(cause) => native_source(cause),
        ProviderExecutionDenial::NativeAdmission(cause) => Cause::NativeAdmission(cause),
        ProviderExecutionDenial::Product(cause) => Cause::Product(cause),
        ProviderExecutionDenial::GraphRead(cause) => graph_read(cause),
        ProviderExecutionDenial::SourceCompletion(cause) => source(cause),
        ProviderExecutionDenial::SourceRead(cause) => {
            use application_query::output_source_observation::SourceQueryObservationStop as Stop;
            match cause {
                Stop::CurrentControl(cause) => Cause::CurrentControl(cause),
                Stop::ManagedIndex(cause) => cause.cause(),
                Stop::Installation(cause) => Cause::Installation(cause),
                Stop::Principal(cause) => Cause::Principal(cause),
                Stop::Scope(cause) => Cause::Scope(cause),
                Stop::Admission(cause) | Stop::Preparation { cause, .. } => {
                    Cause::QueryAdmission(cause)
                }
                Stop::SourceFacts(cause) => Cause::Source(cause),
                Stop::Read(application_query::WorthQueryAdmittedOneShotStop::Execution(cause)) => {
                    Cause::QueryRead(cause)
                }
                Stop::Read(application_query::WorthQueryAdmittedOneShotStop::Admission(cause)) => {
                    Cause::NativeAdmission(cause)
                }
                Stop::Basis(_) => Cause::Refused(Refusal::ForeignBasis),
                Stop::Read(
                    application_query::WorthQueryAdmittedOneShotStop::WorkCounterOverflow
                    | application_query::WorthQueryAdmittedOneShotStop::WorkAccountingMismatch,
                ) => Cause::Refused(Refusal::CapacityAccountingOverflow),
                Stop::Missing | Stop::Ambiguous | Stop::ForeignClosure => {
                    Cause::Refused(Refusal::ForeignContext)
                }
            }
        }
        ProviderExecutionDenial::Execution(cause) => Cause::Execution(cause),
        ProviderExecutionDenial::Capacity(cause) => match cause {
            WorthQueryGraphWorkCapacityAdmissionStop::Admission(cause) => Cause::Execution(cause),
            WorthQueryGraphWorkCapacityAdmissionStop::Denial(cause) => Cause::GraphWork(cause),
            WorthQueryGraphWorkCapacityAdmissionStop::AccountingOverflow => {
                Cause::Refused(Refusal::CapacityAccountingOverflow)
            }
        },
        ProviderExecutionDenial::Identity(cause) => Cause::Refused(match cause {
            WorthQueryManagedGraphWorkSessionStartDenial::PlanAffinityMismatch => {
                Refusal::PlanAffinityMismatch
            }
            WorthQueryManagedGraphWorkSessionStartDenial::BasisBranchMismatch => {
                Refusal::BasisBranchMismatch
            }
            WorthQueryManagedGraphWorkSessionStartDenial::IdentityExhausted => {
                Refusal::SessionIdentityExhausted
            }
            WorthQueryManagedGraphWorkSessionStartDenial::ManagedRunIdentityExhausted => {
                Refusal::ManagedRunIdentityExhausted
            }
        }),
        ProviderExecutionDenial::Session(cause) => session(cause),
        ProviderExecutionDenial::Acquisition(cause) => match &cause.cause {
            AcquisitionCause::ForeignContext => Cause::Refused(Refusal::ForeignContext),
            AcquisitionCause::Execution(cause) => Cause::Execution(cause),
            AcquisitionCause::Principal(cause) => Cause::Principal(cause),
            AcquisitionCause::Scope(cause) => Cause::Scope(cause),
            AcquisitionCause::Product(cause) => Cause::Product(cause),
            AcquisitionCause::Admission(cause) => Cause::QueryAdmission(cause),
            AcquisitionCause::Source(cause) => source(cause),
        },
        ProviderExecutionDenial::Lifecycle(cause) => lifecycle(cause),
        ProviderExecutionDenial::Work(cause) => Cause::Work(cause),
        ProviderExecutionDenial::Selection(cause) => match cause {
            SelectionDenial::Session(cause) => session(cause),
            SelectionDenial::Work(cause) => Cause::Work(cause),
            SelectionDenial::Execution(cause) => Cause::Execution(cause),
            SelectionDenial::ForeignSelection => Cause::Refused(Refusal::ForeignSelection),
        },
        ProviderExecutionDenial::ForeignAssociation => Cause::Refused(Refusal::ForeignAssociation),
        ProviderExecutionDenial::UnpublishedSourceChange => {
            Cause::Refused(Refusal::UnpublishedSourceChange)
        }
    }
}
fn native_source(
    cause: &worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial,
) -> Cause<'_> {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interruption;
    use worth_relational::facade::runtime::{
        CustomInvariantInterruption as Stop, DerivedRelationReadDenial as Read,
        ManagedIndexSourcePreparationDenial as Native, StructuralWalkDenialKind as Structural,
    };
    match cause {
        Native::Source(Read::Request(cause)) => Cause::Execution(cause),
        Native::Source(Read::Structural(Structural::Interrupted(stop))) => {
            Cause::Interrupted(match stop {
                Stop::Cancelled => Interruption::Cancelled,
                Stop::DeadlineElapsed => Interruption::DeadlineExceeded,
            })
        }
        Native::Source(Read::Structural(Structural::MemoryAdmission(cause))) => {
            Cause::Request(worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(*cause),
            ))
        }
        Native::Publication(cause) => Cause::NativeAdmission(cause),
        Native::RootUnavailable => Cause::Refused(Refusal::RootUnavailable),
        cause => Cause::NativeSource(cause),
    }
}
fn source(cause: &SourceDenial) -> Cause<'_> {
    match cause {
        SourceDenial::Admission(cause) => Cause::NativeAdmission(cause),
        SourceDenial::Native(cause) => native_source(cause),
        SourceDenial::Read(cause) => Cause::QueryRead(cause),
        SourceDenial::Work(cause) => Cause::Work(cause),
        SourceDenial::Source(cause) => Cause::Source(cause),
    }
}
pub fn lifecycle(cause: &WorthQueryManagedIndexLifecycleDenial) -> Cause<'_> {
    use WorthQueryManagedIndexLifecycleDenial as Denial;
    match cause {
        Denial::Execution(cause) => Cause::Execution(cause),
        Denial::Session(cause) => session(cause),
        Denial::ForeignAssociation => Cause::Refused(Refusal::ForeignAssociation),
        Denial::RegistrationCapacityExceeded => {
            Cause::Refused(Refusal::RegistrationCapacityExceeded)
        }
        Denial::RegistrationIdentityExhausted => {
            Cause::Refused(Refusal::RegistrationIdentityExhausted)
        }
        Denial::Disposed => Cause::Refused(Refusal::Disposed),
        Denial::BuildInProgress => Cause::Refused(Refusal::BuildInProgress),
        Denial::EpochExhausted => Cause::Refused(Refusal::EpochExhausted),
        Denial::StaleImage => Cause::Refused(Refusal::StaleImage),
        Denial::NotReady => Cause::Refused(Refusal::NotReady),
        Denial::InvariantOwned => Cause::Refused(Refusal::InvariantOwned),
    }
}
fn session(cause: &WorthQueryManagedIndexSessionDenial) -> Cause<'_> {
    use WorthQueryManagedIndexSessionDenial as Denial;
    match cause {
        Denial::Request(cause) => Cause::Request(*cause),
        Denial::Interrupted(cause) => Cause::Interrupted(*cause),
        Denial::Execution(cause) => Cause::Execution(cause),
        Denial::Product(cause) => Cause::Product(cause),
        Denial::AuthorityMismatch => Cause::Refused(Refusal::SessionAuthorityMismatch),
        Denial::ForeignReadIntent => Cause::Refused(Refusal::ForeignReadIntent),
        Denial::Parent(cause) => graph_read(cause),
    }
}
fn graph_read(cause: &WorthQueryManagedGraphReadDenial) -> Cause<'_> {
    match cause {
        WorthQueryManagedGraphReadDenial::RequestStop(cause) => Cause::Execution(cause),
        WorthQueryManagedGraphReadDenial::SnapshotUnavailable => {
            Cause::Refused(Refusal::SnapshotUnavailable)
        }
        WorthQueryManagedGraphReadDenial::IndexNotInstalled(id) => {
            Cause::Refused(Refusal::IndexNotInstalled(*id))
        }
        WorthQueryManagedGraphReadDenial::MutationSession => {
            Cause::Refused(Refusal::MutationSession)
        }
        WorthQueryManagedGraphReadDenial::ForeignBasis => Cause::Refused(Refusal::ForeignBasis),
        WorthQueryManagedGraphReadDenial::ForeignGraph => Cause::Refused(Refusal::ForeignGraph),
        WorthQueryManagedGraphReadDenial::ForeignReadProof => {
            Cause::Refused(Refusal::ForeignReadProof)
        }
        WorthQueryManagedGraphReadDenial::ForeignReadIntent => {
            Cause::Refused(Refusal::ForeignReadIntent)
        }
    }
}

#[cfg(test)]
mod native_source_tests;
