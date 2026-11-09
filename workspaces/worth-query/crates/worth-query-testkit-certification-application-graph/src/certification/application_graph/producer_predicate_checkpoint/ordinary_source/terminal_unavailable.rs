//! Atomic fixed refusal preserves its real producer cause and cleanup custody.
pub use super::*;
pub use worth_query_testkit_document_retention::facade::document_retention_model::retention_entry::{
    assessment_counts, candidate_count, decision_count,
};
pub use std::num::NonZeroUsize;
pub use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::*,
    primary_graph::{
        WorthQueryConditionalRuntimeInstallationDenialKind, WorthQueryOutputDemandDenialKind,
        WorthQueryApplicationUncommitted,
    },
};

pub type Action = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    OrdinaryAssessmentDelivery,
>;
pub type Operation = WorthQueryFixedOperation<
    DocumentRetentionSchema,
    SetRetentionIntent,
    OrdinaryAssessmentProgram,
    OrdinaryRoot,
    Action,
>;
pub type Terminal = WorthQueryTerminalFixedOperation<
    DocumentRetentionSchema,
    SetRetentionIntent,
    OrdinaryAssessmentProgram,
    OrdinaryRoot,
    Action,
>;
pub type Host =
    worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema,
        OrdinaryAssessmentProgram,
    >;

pub fn close_original(host: &Host, terminal: Terminal) -> WorthQueryClosedOperation {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let saved = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let closed = terminal
        .close_with_evidence(&request)
        .unwrap_or_else(|(_, denial)| panic!("atomic refusal cleanup closes: {denial:?}"));
    let Some(WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(refusal),
    )) = closed.source_outcome()
    else {
        panic!("the closed owner preserves the original Native refusal");
    };
    let denial = refusal
        .coupled_producer_denial()
        .expect("actual synchronous producer cause");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert_eq!(
        denial.subject(),
        "worth.query.certification.retention-assessment.producer.v1"
    );
    assert_eq!(
        denial.domain_reason(),
        Some("A zero-day source has no assessable retention period.")
    );
    assert_declared_domain_value(denial);
    assert!(closed.source_result().is_none());
    assert!(
        closed.close_evidence().output_unavailable().is_none(),
        "atomic refusal never claims that a source was saved"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &saved
    );
    closed.into_close_evidence()
}

pub fn install() -> Host {
    publish(
        ApplicationProgramAuthoring::<DocumentRetentionSchema, OrdinaryAssessmentProgram>::begin()
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .unwrap()
}

pub fn terminal_owner(host: &Host) -> Terminal {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let original = request
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
        .unwrap_or_else(|(_, denial)| panic!("ordinary installed leaf: {denial:?}"));
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let (outcome, work) = worth_relational::facade::inspection::observe_maintenance_work(8, || {
        original.advance(&request)
    });
    assert_eq!(
        work.published_rows.len() + work.omitted_publications,
        0,
        "no Native publication on any branch, including a partial source"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_eq!(request.query(DocumentRetentionRead { identity: DOCUMENT_IDENTITY.into() })
        .execute().unwrap().rows()[0].retention_days,
        worth_query_testkit_document_retention::facade::document_retention_model::host::SEED_RETENTION);
    match outcome {
        WorthQueryOperationAdvance::Terminal {
            operation,
            denial: WorthQueryOperationAdvanceDenial::NativeTerminal,
        } => operation,
        _ => panic!("the synchronous producer refusal must refuse the whole edit"),
    }
}

/// The real handler's typed refusal crosses the demand boundary as the
/// provider's declared value: its binding decodes it exactly, a diagnostic
/// copy keeps the same bytes, and any other binding is refused by identity.
pub fn assert_declared_domain_value(
    denial: &worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenial,
) {
    use worth_query_testkit_document_retention::facade::document_retention_model::assessment_output::{
        RetentionAssessmentDenial, RetentionAssessmentDenialValue,
    };
    use worth_query_host::facade::primary_graph::WorthQueryProducerDenialDecodeDenial as Decode;
    assert_eq!(
        denial.decode_producer_denial::<RetentionAssessmentDenialValue>(),
        Ok(RetentionAssessmentDenial::NoRetentionPeriod)
    );
    let bytes = denial
        .producer_denial_bytes()
        .expect("the declared value crosses as canonical bytes");
    assert_eq!(denial.clone().producer_denial_bytes(), Some(bytes));
    assert!(matches!(
        denial.decode_producer_denial::<
            worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding,
        >(),
        Err(Decode::TypeMismatch {
            observed: "worth.query.certification.retention-assessment.denial-value.v1",
            ..
        })
    ));
}

pub fn refuse_foreign_and_cancelled(host: &Host, terminal: Terminal) -> Terminal {
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
pub fn cancelled_scope() -> WorthQueryRequestScope {
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    )
}
pub fn assert_cancelled(denial: WorthQueryOperationCloseDenial) {
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));
}
