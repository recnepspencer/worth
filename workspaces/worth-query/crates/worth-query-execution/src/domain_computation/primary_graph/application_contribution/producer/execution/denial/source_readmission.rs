//! Request rejection remains local to the caller through both query phases.

use super::request_execution::kind as request_execution_kind;
use super::WorthQueryOutputDemandDenialKind as DemandKind;
use super::{
    denial, failed, request_admission_denied, request_admission_rejected, request_authority_stop,
    ProducerExecutionStop,
};
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationOneShotDenial, WorthQueryApplicationOneShotDenialKind as ExecutionKind,
    WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind as AdmissionKind,
};

pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn query_admission_denied(
    subject: &str,
    error: WorthQueryApplicationQueryAdmissionDenial,
) -> ProducerExecutionStop {
    match error.kind() {
        AdmissionKind::ExecutionRequest(cause) => ProducerExecutionStop::ExecutionStopped(denial(
            request_execution_kind(cause),
            format!("{subject}: {error:?}"),
        )),
        AdmissionKind::StalePrincipal
        | AdmissionKind::ForeignPrincipal
        | AdmissionKind::ForeignScope
        | AdmissionKind::StaleScope
        | AdmissionKind::ScopeTypeMismatch => request_admission_denied(subject, error),
        AdmissionKind::Authorization(kind) => {
            request_admission_rejected(request_authority_stop(subject, kind))
        }
        AdmissionKind::Cancelled => {
            request_admission_rejected(denial(DemandKind::Cancelled, subject.to_owned()))
        }
        AdmissionKind::DeadlineExceeded => {
            request_admission_rejected(denial(DemandKind::TimedOut, subject.to_owned()))
        }
        other => {
            let kind = match other {
                AdmissionKind::WorkLimitExceeded => DemandKind::WorkBudgetExceeded,
                AdmissionKind::ReadmissionPreparationMemoryExhausted => {
                    DemandKind::RetentionBudgetExceeded
                }
                _ => DemandKind::ProducerUnavailable,
            };
            ProducerExecutionStop::ExecutionStopped(denial(kind, format!("{subject}: {error:?}")))
        }
    }
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn query_execution_denied(
    subject: &str,
    error: WorthQueryApplicationOneShotDenial,
) -> ProducerExecutionStop {
    match error.kind() {
        ExecutionKind::ExecutionRequest(cause) => ProducerExecutionStop::ExecutionStopped(denial(
            request_execution_kind(cause),
            format!("{subject}: {error:?}"),
        )),
        ExecutionKind::StalePrincipal | ExecutionKind::StaleScope => {
            request_admission_denied(subject, error)
        }
        ExecutionKind::Authorization(kind) => {
            request_admission_rejected(request_authority_stop(subject, kind))
        }
        ExecutionKind::Cancelled => {
            request_admission_rejected(denial(DemandKind::Cancelled, subject.to_owned()))
        }
        ExecutionKind::DeadlineExceeded => {
            request_admission_rejected(denial(DemandKind::TimedOut, subject.to_owned()))
        }
        _ => ProducerExecutionStop::ExecutionStopped(failed(subject, error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryAdvancementDenial as Stop,
        WorthQueryApplicationAttemptDenial, WorthQueryInvariantProjectionDenialKind as Invariant,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryOperationProjectionDenialKind as Projection,
    };
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Native;

    #[test]
    fn carried_native_read_stop_survives_producer_admission_and_body_receivers() {
        let expected = DemandKind::ExecutionRequest(Stop::Resource(Resource::WorkExhausted));
        let native = Native::ExecutionResource {
            denial: Resource::WorkExhausted,
            partition_identity: None,
            policy_ancestor: None,
        };
        let admission = WorthQueryApplicationQueryAdmissionDenial::new(
            AdmissionKind::ExecutionRequest(native),
            "actual source read",
        );
        let ProducerExecutionStop::ExecutionStopped(stopped) =
            query_admission_denied("producer", admission)
        else {
            panic!("resource stop must not be an authority rejection");
        };
        assert_eq!(stopped.kind(), expected);
        assert_eq!(super::super::projection_budget(Projection::InvariantAdmission(
            Invariant::ExecutionRequest(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted))), Some(expected.clone()));
        let attempt = WorthQueryApplicationAttemptDenial::relational_read_stopped(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted,
            "body",
        );
        let ProducerExecutionStop::ExecutionStopped(stopped) = super::super::execution_failed(
            "producer",
            MutationHandlerExecutionDenial::Attempt(attempt),
        ) else {
            panic!("body resource stop remains with execution");
        };
        assert_eq!(stopped.kind(), expected);
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeStop;
        for (cause, expected) in [
            (NativeStop::Cancelled, DemandKind::Cancelled),
            (NativeStop::DeadlineElapsed, DemandKind::TimedOut),
        ] {
            let attempt =
                WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, "body");
            let ProducerExecutionStop::RequestAdmissionDenied(rejected) =
                super::super::execution_failed(
                    "producer",
                    MutationHandlerExecutionDenial::Attempt(attempt),
                )
            else {
                panic!("body control stop must stay with the request");
            };
            assert_eq!(rejected.into_denial().kind(), expected);
        }
    }

    #[test]
    fn attributed_native_stop_and_genuine_absent_handler_keep_distinct_causes() {
        let native = Native::ExecutionResource {
            denial: Resource::WorkExhausted,
            partition_identity: Some(17),
            policy_ancestor: Some(2),
        };
        assert_eq!(
            request_execution_kind(native),
            DemandKind::SourceQueryExecution(ExecutionKind::ExecutionRequest(native))
        );
        let ProducerExecutionStop::ExecutionStopped(stopped) = super::super::execution_failed(
            "producer",
            MutationHandlerExecutionDenial::HandlerNotInstalled,
        ) else {
            panic!("an actually absent handler remains a producer failure");
        };
        assert_eq!(stopped.kind(), DemandKind::ProducerUnavailable);
    }
    #[test]
    fn unpartitioned_kernel_stops_stay_request_local_across_source_and_commit() {
        use crate::domain_computation::primary_graph::{
            application_attempt::provider_compare_denial::provider_session_execution_denied,
            WorthQueryApplicationCommitDenialStage as Stage,
            WorthQueryApplicationCommitOutcome as Outcome,
        };
        for (native, expected) in [
            (
                Native::ExecutionNestedPatternStopped {
                    partition_identity: None,
                },
                DemandKind::ExecutionRequest(Stop::NestedStopped),
            ),
            (
                Native::ExecutionWorkerPanicked {
                    partition_identity: None,
                },
                DemandKind::ExecutionRequest(Stop::Panicked),
            ),
        ] {
            let source = query_admission_denied(
                "producer",
                WorthQueryApplicationQueryAdmissionDenial::new(
                    AdmissionKind::ExecutionRequest(native),
                    "source request executor stopped",
                ),
            )
            .into_output_denial();
            let commit = provider_session_execution_denied(
                Ok(native),
                Stage::ProviderCommit,
                "commit request executor stopped",
            );
            let commit = crate::domain_computation::primary_graph::application_contribution::producer::execution::commit_receipt("producer", Outcome::Denied(commit)).err().expect("native commit stopped");
            assert_eq!(source.kind(), expected);
            assert_eq!(commit.kind(), expected);
            assert_eq!(
                commit.commit_execution_denial(),
                Some((Stage::ProviderCommit, Ok(native)))
            );
        }
    }
}
