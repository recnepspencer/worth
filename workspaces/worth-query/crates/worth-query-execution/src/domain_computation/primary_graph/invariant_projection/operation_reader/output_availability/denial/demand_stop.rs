//! Existing output stop vocabulary, retaining request/control/resource posture.
use super::*;
use crate::domain_computation::primary_graph::output_reuse;
use crate::domain_computation::primary_graph::{
    application_query::{
        SourceQueryObservationStop as QueryStop, WorthQueryAdmittedOneShotStop as ReadStop,
    },
    WorthQueryApplicationOneShotDenialKind as ReadKind,
    WorthQueryApplicationQueryAdmissionDenialKind as AdmissionKind,
    WorthQueryCurrentOutputDenialKind as CurrentKind, WorthQueryOutputDemandDenialKind as Kind,
};

impl WorthQueryOutputAvailabilityDenial {
    /// Native refusal classification; diagnostic state grants no availability.
    pub fn kind(&self) -> Kind {
        self.demand_stop().kind()
    }

    pub(in crate::domain_computation::primary_graph) fn demand_stop(
        &self,
    ) -> WorthQueryOutputDemandDenial {
        use Cause::*;
        let kind = match self.cause.as_ref() {
            Verification(stop) => {
                return output_reuse::verification_denial(*stop, "output availability")
            }
            Registration(SettlementRegistrationStop::Admission(stop)) => return meter(*stop),
            Registration(SettlementRegistrationStop::Edit(_)) => Kind::PublicationStale,
            Registration(_) => Kind::RetainedBasisUnavailable,
            Producer(stop) => return stop.clone(),
            ContextUnavailable | Decision(_) => Kind::ProducerUnavailable,
            SourceMismatch => Kind::ForeignSource,
            SourceQuery(stop) => return query_stop(stop),
            Current(stop) => match stop.kind() {
                CurrentKind::ExecutionRequest(cause) => return output_reuse::read_denial(cause),
                CurrentKind::SourceQueryAuthorization(kind) => Kind::of_request_authorization(kind),
                CurrentKind::WorkBudgetExceeded => Kind::WorkBudgetExceeded,
                CurrentKind::RetentionCapacityExhausted => Kind::RetentionBudgetExceeded,
                CurrentKind::Interrupted(interruption) => Kind::of_interruption(interruption),
                CurrentKind::CurrentnessRaced(_) => Kind::PublicationStale,
                _ => Kind::RetainedBasisUnavailable,
            },
        };
        WorthQueryOutputDemandDenial::new(kind, format!("{self}"))
    }
}

fn meter(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    output_reuse::retained_fact_denial(output_reuse::preflight_stop(stop), "output availability")
}

fn query_stop(stop: &QueryStop) -> WorthQueryOutputDemandDenial {
    use crate::domain_computation::primary_graph::product_operation::SelectedQueryBasisRetentionStop as Basis;

    let kind = match stop {
        QueryStop::Installation(stop) => Kind::SourceQueryInstallation(stop.kind()),
        QueryStop::Principal(stop) => Kind::of_principal_resolution(stop.kind()),
        QueryStop::Scope(stop) => Kind::of_scope_resolution(stop.kind()),
        QueryStop::Admission(stop) | QueryStop::Preparation { cause: stop, .. } => {
            match stop.kind() {
                AdmissionKind::Cancelled => Kind::Cancelled,
                AdmissionKind::DeadlineExceeded => Kind::TimedOut,
                AdmissionKind::Authorization(kind) => Kind::of_request_authorization(kind),
                AdmissionKind::WorkLimitExceeded => Kind::WorkBudgetExceeded,
                AdmissionKind::ReadmissionPreparationMemoryExhausted => {
                    Kind::RetentionBudgetExceeded
                }
                kind => Kind::SourceQueryAdmission(kind),
            }
        }
        QueryStop::Read(ReadStop::Admission(stop)) | QueryStop::Basis(Basis::Admission(stop)) => {
            return meter(*stop)
        }
        QueryStop::Read(ReadStop::Execution(stop)) => match stop.kind() {
            ReadKind::Cancelled => Kind::Cancelled,
            ReadKind::DeadlineExceeded => Kind::TimedOut,
            ReadKind::Authorization(kind) => Kind::of_request_authorization(kind),
            ReadKind::WorkLimitExceeded => Kind::WorkBudgetExceeded,
            kind => Kind::SourceQueryExecution(kind),
        },
        QueryStop::Read(_) => Kind::WorkBudgetExceeded,
        QueryStop::Basis(Basis::Basis) => Kind::RetainedBasisUnavailable,
        QueryStop::Missing | QueryStop::Ambiguous => {
            Kind::SourceQueryExecution(ReadKind::CardinalityMismatch)
        }
        QueryStop::ForeignClosure => Kind::ForeignSource,
        QueryStop::SourceFacts(stop) => {
            if let Some(interruption) = stop.retention_interruption() {
                return WorthQueryOutputDemandDenial::new(match interruption {
                    worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => Kind::Cancelled,
                    worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => Kind::TimedOut,
                }, "source query fact retention");
            }
            use crate::domain_computation::primary_graph::WorthQuerySourceExpectationDenialKind as Source;
            match stop.kind() {
                Source::WorkBudgetExceeded => Kind::WorkBudgetExceeded,
                Source::PreparationMemoryExceeded | Source::RetainedSourceStorageDenied => {
                    Kind::RetentionBudgetExceeded
                }
                _ => Kind::ForeignSource,
            }
        }
    };
    WorthQueryOutputDemandDenial::new(kind, format!("source query observation: {stop:?}"))
}
