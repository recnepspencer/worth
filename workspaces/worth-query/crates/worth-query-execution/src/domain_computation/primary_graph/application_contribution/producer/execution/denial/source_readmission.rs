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
    if admission_authority_rejection(error.kind()) {
        return match error.kind() {
            AdmissionKind::Authorization(kind) => {
                request_admission_rejected(request_authority_stop(subject, kind))
            }
            _ => request_admission_denied(subject, error),
        };
    }
    match error.kind() {
        AdmissionKind::ExecutionRequest(cause) => ProducerExecutionStop::ExecutionStopped(denial(
            request_execution_kind(cause),
            format!("{subject}: {error:?}"),
        )),
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
    if execution_authority_rejection(error.kind()) {
        return match error.kind() {
            ExecutionKind::Authorization(kind) => {
                request_admission_rejected(request_authority_stop(subject, kind))
            }
            _ => request_admission_denied(subject, error),
        };
    }
    match error.kind() {
        ExecutionKind::ExecutionRequest(cause) => ProducerExecutionStop::ExecutionStopped(denial(
            request_execution_kind(cause),
            format!("{subject}: {error:?}"),
        )),
        ExecutionKind::Cancelled => {
            request_admission_rejected(denial(DemandKind::Cancelled, subject.to_owned()))
        }
        ExecutionKind::DeadlineExceeded => {
            request_admission_rejected(denial(DemandKind::TimedOut, subject.to_owned()))
        }
        _ => ProducerExecutionStop::ExecutionStopped(failed(subject, error)),
    }
}

/// Selected Source disclosure preserves the original admitted read or managed cause.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn selected_source_read_denied(
    subject: &'static str,
    stop: crate::domain_computation::primary_graph::application_query::SourceQueryObservationStop,
) -> ProducerExecutionStop {
    use crate::domain_computation::primary_graph::application_query::{
        SourceQueryObservationStop as Stop, WorthQueryAdmittedOneShotStop as Read,
    };
    match stop {
        Stop::CurrentControl(stop) => request_admission_rejected(stop),
        Stop::Read(Read::Admission(stop)) => {
            super::super::source_readmission::ready_resource_denial(subject, stop)
        }
        Stop::Read(Read::WorkCounterOverflow | Read::WorkAccountingMismatch) => {
            ProducerExecutionStop::ExecutionStopped(denial(DemandKind::WorkBudgetExceeded, subject))
        }
        Stop::Read(Read::Execution(stop)) => query_execution_denied(subject, stop),
        Stop::ManagedIndex(cause) => {
            let caller_local = managed_authority_rejection(cause.cause());
            let denial = super::WorthQueryOutputDemandDenial::managed_source_query(cause, subject);
            if caller_local || matches!(denial.kind(), DemandKind::Cancelled | DemandKind::TimedOut)
            {
                request_admission_rejected(denial)
            } else {
                ProducerExecutionStop::ExecutionStopped(denial)
            }
        }
        other => ProducerExecutionStop::ExecutionStopped(failed(subject, other)),
    }
}

fn managed_authority_rejection(
    cause: crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexDenialCause<'_>,
) -> bool {
    use crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexDenialCause as Cause;
    match cause {
        Cause::CurrentControl(_) => true,
        Cause::Principal(_) | Cause::Scope(_) => true,
        Cause::QueryAdmission(stop) => admission_authority_rejection(stop.kind()),
        Cause::QueryRead(stop) => execution_authority_rejection(stop.kind()),
        _ => false,
    }
}

fn admission_authority_rejection(kind: AdmissionKind) -> bool {
    matches!(
        kind,
        AdmissionKind::Authorization(_)
            | AdmissionKind::StalePrincipal
            | AdmissionKind::ForeignPrincipal
            | AdmissionKind::ForeignScope
            | AdmissionKind::StaleScope
            | AdmissionKind::ScopeTypeMismatch
    )
}
fn execution_authority_rejection(kind: ExecutionKind) -> bool {
    matches!(
        kind,
        ExecutionKind::Authorization(_) | ExecutionKind::StalePrincipal | ExecutionKind::StaleScope
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_control_stays_caller_local_while_intrinsic_managed_refusals_do_not() {
        use crate::domain_computation::primary_graph::{
            application_query::{
                SourceQueryObservationStop as Stop, WorthQueryManagedIndexDenialCause as Cause,
                WorthQueryManagedIndexRefusal as Refusal,
            },
            WorthQueryOutputDemandDenial, WorthQueryOutputDemandRecoveryPosture as Posture,
        };
        let original =
            WorthQueryOutputDemandDenial::new(DemandKind::CurrentControlBusy, "current control");
        assert!(managed_authority_rejection(Cause::CurrentControl(
            &original
        )));
        assert!(!managed_authority_rejection(Cause::Refused(
            Refusal::ForeignContext
        )));
        let ProducerExecutionStop::RequestAdmissionDenied(rejected) =
            selected_source_read_denied("producer", Stop::CurrentControl(original))
        else {
            panic!("current control must not fail a shared output");
        };
        let denial = rejected.into_denial();
        assert_eq!(denial.kind(), DemandKind::CurrentControlBusy);
        assert_eq!(denial.recovery_posture(), Posture::Retryable);
    }
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
            let Err(commit) = crate::domain_computation::primary_graph::application_contribution::producer::execution::commit_receipt("producer", Outcome::Denied(commit)) else {
                panic!("native commit unexpectedly succeeded");
            };
            assert_eq!(source.kind(), expected);
            assert_eq!(commit.kind(), expected);
            assert_eq!(
                commit.commit_execution_denial(),
                Some((Stage::ProviderCommit, Ok(native)))
            );
        }
    }
    #[test]
    fn managed_authority_diagnostics_keep_the_ordinary_caller_local_boundary() {
        use crate::domain_computation::authorization::{
            WorthQueryOperationAuthorizationDenialKind as Auth,
            WorthQueryOperationAuthorizationRefusalKind as Refused,
        };
        for kind in [
            AdmissionKind::StalePrincipal,
            AdmissionKind::ForeignPrincipal,
            AdmissionKind::ForeignScope,
            AdmissionKind::StaleScope,
            AdmissionKind::ScopeTypeMismatch,
            AdmissionKind::Authorization(Auth::Refused(Refused::StaleAuthorization)),
        ] {
            assert!(admission_authority_rejection(kind));
        }
        for kind in [
            ExecutionKind::StalePrincipal,
            ExecutionKind::StaleScope,
            ExecutionKind::Authorization(Auth::Refused(Refused::StaleAuthorization)),
        ] {
            assert!(execution_authority_rejection(kind));
        }
        assert!(!admission_authority_rejection(
            AdmissionKind::WorkLimitExceeded
        ));
        assert!(!execution_authority_rejection(
            ExecutionKind::WorkLimitExceeded
        ));
    }
}
