//! Real owner refusals cross both production completion consumer seams.
use super::fixture::installed_world;
use crate::domain_computation::application_aftermath::{
    dispatch_external_effect, WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};
use crate::domain_computation::primary_graph::application_runtime::InstalledTransportPendingReason as Pending;
use crate::domain_computation::primary_graph::application_runtime::{
    authenticated_completion_candidate, transport_completion_candidate,
    InstalledTransportPublicationOutcome as TransportOutcome,
    WorthQueryInboundPublicationDenial as PublicationDenial,
    WorthQueryInboundPublicationOutcome as PublicationOutcome,
};
use crate::domain_computation::primary_graph::bootstrap_publication::{
    commit_refusals::{refused_commit, work_refusal},
    execution_refusals::{isolated, request},
};
use crate::domain_computation::primary_graph::provider::{
    completion_preparation_denial, completion_validation_denial,
    WorthQueryInboundCompletionPreparationDenial as Preparation,
};
use crate::domain_computation::primary_graph::{
    InstalledTransportCompletion, WorthQueryApplicationCommitDenialStage as Stage,
    WorthQueryInboundAdmissionDenial as Retry, WorthQueryInboundPendingReason as ReceiptPending,
    WorthQueryManagedComputationResourceDenial as Resource,
};
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind,
};
use std::sync::Arc;
use std::time::Instant;
use worth_execution::CancellationSource;

struct CompletingTransport;
impl WorthQueryExternalEffectTransport for CompletingTransport {
    fn dispatch(
        &self,
        _: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        WorthQueryExternalTransportOutcome::Completed
    }
}

#[test]
fn inbound_publication_consumers_keep_real_execution_refusals_and_retry_posture() {
    isolated(
        concat!(
            module_path!(),
            "::inbound_publication_consumers_keep_real_execution_refusals_and_retry_posture"
        ),
        || {
            // No lease is threaded into production here. The candidate-result seam
            // receives a real Relational refusal; transport evidence is owner-issued.
            let world = installed_world();
            let receipt = world.commit_dispatch(71, "completion-execution-refusal");
            let owner = world
                .application
                .observe_committed_dispatch_outbox(&receipt)
                .unwrap()
                .unwrap();
            let admitted = world
                .application
                .admit_external_dispatch_attempt(owner.clone())
                .unwrap();
            let completed = dispatch_external_effect(&CompletingTransport, admitted).unwrap();
            let evidence = Arc::new(
                InstalledTransportCompletion::from_observed_dispatch(owner, &completed).unwrap(),
            );
            let source = CancellationSource::new();
            source.cancel();
            let mut canceled = request();
            canceled.cancellation = source.token();
            let mut expired = request();
            expired.deadline = Some(Instant::now());
            let refusals = [
                (
                    work_refusal(),
                    Ok(Kind::ExecutionResource {
                        denial: Resource::WorkExhausted,
                        partition_identity: Some(1),
                        policy_ancestor: None,
                    }),
                ),
                (refused_commit(canceled), Err(Control::Cancelled)),
                (refused_commit(expired), Err(Control::TimedOut)),
            ];
            for (error, expected) in refusals {
                for (preparation, stage) in [
                    (completion_preparation_denial(&error), Stage::ProviderCommit),
                    (
                        completion_validation_denial(&error),
                        Stage::InvariantExecution,
                    ),
                ] {
                    let expected_preparation = match expected {
                        Ok(kind) => Preparation::ExecutionDenied { stage, kind },
                        Err(kind) => Preparation::ExecutionControlStopped { stage, kind },
                    };
                    assert_eq!(preparation, expected_preparation);
                    let Err(PublicationOutcome::Denied(PublicationDenial::CompletionPreparation(
                        observed,
                    ))) = authenticated_completion_candidate(Err(preparation))
                    else {
                        panic!("authenticated publication consumer discarded the owner refusal");
                    };
                    assert_eq!(observed, expected_preparation);
                    let observed =
                        PublicationDenial::CompletionPreparation(observed).retry_denial();
                    match (expected, observed.pending_reason()) {
                        (
                            Ok(kind),
                            ReceiptPending::ExecutionDenied {
                                stage: actual,
                                kind: cause,
                            },
                        ) => {
                            assert_eq!(actual, stage);
                            assert_eq!(cause, kind);
                        }
                        (
                            Err(kind),
                            ReceiptPending::ExecutionControlStopped {
                                stage: actual,
                                kind: cause,
                            },
                        ) => {
                            assert_eq!(actual, stage);
                            assert_eq!(cause, kind);
                        }
                        _ => panic!("accepted receipt folded the cause"),
                    }
                    match (expected, observed) {
                        (
                            Ok(kind),
                            Retry::PublicationExecutionDenied {
                                stage: actual,
                                kind: cause,
                            },
                        ) => {
                            assert_eq!(actual, stage);
                            assert_eq!(cause, kind);
                        }
                        (
                            Err(kind),
                            Retry::PublicationExecutionControlStopped {
                                stage: actual,
                                kind: cause,
                            },
                        ) => {
                            assert_eq!(actual, stage);
                            assert_eq!(cause, kind);
                        }
                        _ => panic!("authenticated retry consumer folded the cause"),
                    }
                    let Err(TransportOutcome::Denied(retained, denial)) =
                        transport_completion_candidate(Arc::clone(&evidence), Err(preparation))
                    else {
                        panic!("transport publication consumer discarded the owner refusal");
                    };
                    assert!(
                        Arc::ptr_eq(&retained, &evidence),
                        "real dispatch evidence changed"
                    );
                    let pending = denial.pending_reason();
                    let recovery = crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial::from(
                        pending.dispatch_preparation_denial().redispatch_denial(),
                    ).kind();
                    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenialKind as Recovery;
                    match (expected, recovery) {
                        (
                            Ok(kind),
                            Recovery::CompletionExecutionDenied {
                                stage: actual,
                                kind: cause,
                            },
                        ) => {
                            assert_eq!((actual, cause), (stage, kind));
                        }
                        (
                            Err(kind),
                            Recovery::CompletionExecutionControlStopped {
                                stage: actual,
                                kind: cause,
                            },
                        ) => {
                            assert_eq!((actual, cause), (stage, kind));
                        }
                        _ => panic!("redispatch recovery consumer folded the cause"),
                    }
                    match (expected, pending) {
                        (
                            Ok(kind),
                            Pending::ExecutionDenied {
                                stage: actual,
                                kind: cause,
                            },
                        ) => {
                            assert_eq!(actual, stage);
                            assert_eq!(cause, kind);
                        }
                        (
                            Err(kind),
                            Pending::ExecutionControlStopped {
                                stage: actual,
                                kind: cause,
                            },
                        ) => {
                            assert_eq!(actual, stage);
                            assert_eq!(cause, kind);
                        }
                        _ => panic!("transport retry consumer folded the cause"),
                    }
                }
            }
        },
    );
}
