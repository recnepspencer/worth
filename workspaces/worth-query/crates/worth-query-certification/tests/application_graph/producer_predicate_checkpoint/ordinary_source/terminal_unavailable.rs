//! Ordinary fixed-leaf negative completion retains source truth and cleanup custody.
use super::*;
use crate::document_retention_model::retention_entry::{
    assessment_counts, candidate_count, decision_count,
};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::*,
    primary_graph::{
        WorthQueryConditionalRuntimeInstallationDenialKind, WorthQueryOutputDemandDenialKind,
    },
};

type Action = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    OrdinaryAssessmentDelivery,
>;
type Operation = WorthQueryFixedOperation<
    DocumentRetentionSchema,
    SetRetentionIntent,
    OrdinaryAssessmentProgram,
    OrdinaryRoot,
    Action,
>;
type Terminal = WorthQueryTerminalFixedOperation<
    DocumentRetentionSchema,
    SetRetentionIntent,
    OrdinaryAssessmentProgram,
    OrdinaryRoot,
    Action,
>;
type Host = worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
    DocumentRetentionSchema,
    OrdinaryAssessmentProgram,
>;

#[test]
fn fixed_leaf_domain_refusal_closes_unavailable_without_reexecution() {
    let source_before = (decision_count(0), candidate_count(0));
    let assessment_before = assessment_counts(0);
    let mut host = install();
    let terminal = terminal_owner(&host);
    assert_eq!(host.retained_source_custody_count_for_test(), 1);
    assert_eq!(
        host.close_conditional_runtime().unwrap_err().kind(),
        WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained
    );
    let terminal = refuse_foreign_and_cancelled(&host, terminal);
    assert!(terminal.output_unavailable().is_none());
    let closed = close_original(&host, terminal);
    assert!(closed.output_unavailable().is_some());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    assert_eq!(
        request
            .query(DocumentRetentionRead {
                identity: DOCUMENT_IDENTITY.into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        0,
        "source is independently saved despite no assessment output"
    );
    assert_eq!(decision_count(0), source_before.0 + 1);
    assert_eq!(candidate_count(0), source_before.1 + 1);
    assert_eq!(
        assessment_counts(0),
        (assessment_before.0 + 1, assessment_before.1),
        "exactly one real producer decision, zero output candidates or replay"
    );
    assert_eq!(host.retained_source_custody_count_for_test(), 0);
    assert_eq!(host.prepared_required_output_source_count_for_test(), 0);
    assert_eq!(host.active_application_attempts_for_test(), 0);
    assert_eq!(host.world_active_publication_attempts_for_test(), 0);
    assert!(host
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    host.close_conditional_runtime()
        .expect("lawful operation close released its authentic runtime keepalive");
}

fn close_original(host: &Host, terminal: Terminal) -> WorthQueryClosedOperation {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let saved = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let closed = terminal
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("ordinary negative completion closes: {denial:?}"));
    let unavailable = closed
        .output_unavailable()
        .expect("ordinary close returns genuine negative completion, never settled output");
    assert_eq!(
        unavailable.denial().kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert_eq!(
        unavailable
            .source_receipt()
            .committed_product_publication()
            .composite_commit(),
        &saved,
        "negative completion carries the actual saved source commit"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &saved,
        "negative completion and cleanup publish no assessment result"
    );
    closed
}

fn install() -> Host {
    publish(
        ApplicationProgramAuthoring::<DocumentRetentionSchema, OrdinaryAssessmentProgram>::begin()
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .unwrap()
}

fn terminal_owner(host: &Host) -> Terminal {
    let original = {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        host.runtime()
            .request(&principal, &scope)
            .mutate(SetRetentionIntent {
                input: SetRetentionInput {
                    identity: DOCUMENT_IDENTITY.into(),
                    retention_days: 0,
                },
            })
            .without_source()
            .idempotency_owned(0x9176_6600_u64)
            .unwrap_or_else(|(_, _, denial)| panic!("owning capture: {denial:?}"))
            .retain_operation::<OrdinaryAssessmentProgram, Action>(host)
            .unwrap_or_else(|(_, denial)| panic!("ordinary installed leaf: {denial:?}"))
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let mut original: Operation = match original.advance(&request) {
        WorthQueryOperationAdvance::Progress {
            operation,
            progress,
        } => {
            assert!(matches!(
                progress,
                WorthQueryOperationProgress::SourcePerformed
            ));
            operation
        }
        _ => panic!("real source must enter its performed native owner"),
    };
    let (returned, denial) = original
        .close(&request)
        .err()
        .expect("pending preparation/partial is not negative completion");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::NativeObligationRetained
    ));
    original = returned;
    loop {
        match original.advance(&request) {
            WorthQueryOperationAdvance::Progress { operation, .. } => original = operation,
            WorthQueryOperationAdvance::Terminal {
                operation,
                denial:
                    WorthQueryOperationAdvanceDenial::Outputs(
                        WorthQueryRequiredOutputPreparationDenial::Demand(
                            WorthQueryApplicationOutputDemandDenial::Demand(denial),
                        ),
                    ),
            } => {
                assert_eq!(
                    denial.kind(),
                    WorthQueryOutputDemandDenialKind::ProducerDomainDenied
                );
                assert_eq!(
                    denial.domain_reason(),
                    Some("A zero-day source has no assessable retention period.")
                );
                assert_eq!(denial.recovery_posture(), worth_query_host::facade::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal);
                return operation;
            }
            WorthQueryOperationAdvance::Refused { denial, .. }
            | WorthQueryOperationAdvance::Terminal { denial, .. } => {
                panic!("not the actual producer domain refusal: {denial:?}")
            }
            WorthQueryOperationAdvance::Settled(_) => {
                panic!("zero-day retention cannot publish an assessment result")
            }
        }
    }
}

fn refuse_foreign_and_cancelled(host: &Host, terminal: Terminal) -> Terminal {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let other = host
        .runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .fork(host.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let (terminal, denial) = terminal
        .close(&host.runtime().request(&principal, &scope).on_branch(other))
        .err()
        .expect("foreign branch preserves original owner");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::FreshRequestMismatch
    ));
    let foreign = install();
    let foreign_principal = authenticate_operator(foreign.installed_schema(), &scope);
    let (terminal, denial) = terminal
        .close(&foreign.runtime().request(&foreign_principal, &scope))
        .err()
        .expect("foreign runtime preserves original owner");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::FreshRequestMismatch
    ));
    let stopped = cancelled_scope();
    let (terminal, denial) = terminal
        .close(&host.runtime().request(&principal, &stopped))
        .err()
        .expect("cancelled phase preserves original owner");
    assert_cancelled(denial);
    terminal
}
fn cancelled_scope() -> WorthQueryRequestScope {
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    WorthQueryRequestScope::new(None, cancellation.token())
}
fn assert_cancelled(denial: WorthQueryOperationCloseDenial) {
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));
}
