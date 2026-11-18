use super::*;
use std::sync::atomic::Ordering;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    primary_graph::{
        WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
        WorthQueryApplicationUncommitted, WorthQueryOutputDemandDenialKind,
        WorthQueryOutputDemandRecoveryPosture,
    },
    product::{
        WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchAdmissionRefusalKind,
    },
};

#[test]
fn unavailable_fixed_output_closes_its_original_native_owner_before_candidate_disposal() {
    let world = install();
    let application = &world.application;
    // Read the source model before any output exists. The installed provider's
    // ordinary VerifyCurrentOutputs operation therefore denies in the actual
    // handler, rather than superseding a warmed output while refreshing itself.
    let document=application.with_host_advancement(|phase|{
        let selected=application.runtime().on_branch(application.current_world()).select(&phase).unwrap();
        let candidate=application.runtime().branches().fork_expected(selected).unwrap()
            .components(|parts|parts.fork_relational().reuse_exact_signal_basis()).create(&phase).unwrap();
        let head=candidate.into_owned_head();
        with_branch_request(application,head.branch(),|request|request.attach_history::<worth_query_topology_entry::PlanarSourceDocumentHistory<ConsumerSchema>>(
            &phase,head,PlanarRead{body_key:"anchor-a".into()})
            .unwrap_or_else(|failure|{let (_,cause)=failure.into_parts();panic!("source model attach: {cause:?}")}))
    }).unwrap();
    let before = with_request(application, |request| {
        document.read(request).unwrap().rows().to_vec()
    });
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].y, length(1));
    let selection = with_request(application, |request| document.selection(request).unwrap());
    let source_branch = selection.physical_endpoint().product_branch();
    world.producer_domain_denial.store(true, Ordering::SeqCst);
    let mut edit = with_request(application, |request| {
        document
            .edit(PlanarInitialAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: length(2),
            })
            .idempotency_owned(request, 0x9182_d200)
            .unwrap_or_else(|_| panic!("capture source edit"))
            .retain_in_program::<HistoryProgram, HistoryAction>(application)
            .unwrap_or_else(|(_, cause)| panic!("source edit construction: {cause:?}"))
    });
    let mut candidate = None;
    for _ in 0..64 {
        if let Some(operation) = edit.source_operation() {
            if operation.branch() != source_branch {
                assert!(operation.prepared_source_receipt().is_none());
                candidate = Some(operation.branch());
                break;
            }
        }
        edit = match with_request(application, |request| edit.advance(request)) {
            WorthQueryHistoryEditAdvance::Progress(next)
            | WorthQueryHistoryEditAdvance::Ready(next) => next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("source candidate binding: {denial:?}")
            }
        };
    }
    let candidate = candidate.expect("the original source binds its issued candidate");
    let (candidate_head, output_before) = with_branch_request(application, candidate, |request| {
        let rows = request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap()
            .rows()
            .to_vec();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].body_key, "anchor-a");
        assert_eq!(rows[0].successor_body_key, "anchor-b");
        assert_eq!(rows[0].value, length(1));
        (
            request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone(),
            rows,
        )
    });
    for _ in 0..64 {
        if edit.posture() == WorthQueryHistoryEditPosture::Terminal {
            break;
        }
        edit = match with_request(application, |request| edit.advance(request)) {
            WorthQueryHistoryEditAdvance::Progress(next)
            | WorthQueryHistoryEditAdvance::Ready(next) => next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                panic!("source edit progression: {denial:?}")
            }
        };
    }
    assert_eq!(edit.posture(), WorthQueryHistoryEditPosture::Terminal);
    assert!(edit.selected_step().is_none());
    assert!(edit.completed_source().is_none());
    assert_domain_denial(
        edit.terminal_source().unwrap().advance_denial(),
        edit.terminal_source().unwrap().native_outcome(),
    );
    with_branch_request(application, candidate, |request| {
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &candidate_head
        );
        assert_eq!(
            request
                .query(PlanarRead {
                    body_key: "anchor-a".into()
                })
                .execute()
                .unwrap()
                .rows(),
            before.as_slice()
        );
        assert_eq!(
            request
                .query(PlanarOutputRead {
                    body_key: "anchor-a".into()
                })
                .execute()
                .unwrap()
                .rows(),
            output_before.as_slice()
        );
    });
    let cancellation=worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let adapter = authentication::admit(application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let (edit, denial) = edit
        .finish(&application.request(&principal, &stopped))
        .err()
        .expect("stopped close retains original terminal custody");
    assert!(matches!(
        denial,
        WorthQueryHistoryEditDenial::Close(WorthQueryOperationCloseDenial::Execution(
            WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            )
        ))
    ));
    assert_domain_denial(
        edit.terminal_source().unwrap().advance_denial(),
        edit.terminal_source().unwrap().native_outcome(),
    );
    assert!(edit.closed_terminal().is_none());
    with_branch_request(application, candidate, |request| {
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &candidate_head
        );
    });
    let finished = with_request(application, |request| {
        edit.finish(request).unwrap_or_else(|(edit, cause)| {
            panic!(
                "original domain-refused close: {cause:?}; live={:?}; retired={:?}",
                edit.candidate_close_denial(),
                edit.candidate_cleanup_failure()
            )
        })
    });
    let WorthQueryFixedHistoryEditOutcome::NotRecorded { terminal } = finished.outcome else {
        panic!("failed output must not become a completed history node")
    };
    assert_domain_denial(terminal.advance_denial(), terminal.source_outcome());
    assert!(
        terminal.source_result().is_none(),
        "synchronous refusal did not perform the source"
    );
    assert!(
        terminal.close_evidence().output_unavailable().is_none(),
        "atomic refusal has no saved-source unavailable proof"
    );
    let document = finished.document;
    assert_eq!(
        with_request(application, |request| document
            .read(request)
            .unwrap()
            .rows()
            .to_vec()),
        before
    );
    assert_eq!(
        with_request(application, |request| document.selection(request).unwrap()),
        selection
    );
    let refusal = application
        .with_host_advancement(|phase| {
            match application.runtime().on_branch(candidate).select(&phase) {
                Err(refusal) => refusal,
                Ok(_) => panic!("the closed original candidate must retire"),
            }
        })
        .unwrap();
    assert_eq!(
        refusal,
        WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch
        )
    );
    assert!(application
        .runtime()
        .branches()
        .pending_cleanup()
        .is_empty());
    drop(terminal);
    application
        .with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

fn assert_domain_denial<D, R>(
    denial: Option<&WorthQueryOperationAdvanceDenial>,
    outcome: Option<&WorthQueryApplicationMutationOutcome<D, R>>,
) {
    assert!(
        matches!(
            denial,
            Some(WorthQueryOperationAdvanceDenial::NativeTerminal)
        ),
        "the original native refusal remains on the same terminal owner: {denial:?}"
    );
    let Some(WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(refusal),
    )) = outcome
    else {
        panic!("the actual synchronous source Commit/Denied must survive close")
    };
    assert_eq!(
        refusal.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        refusal.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let cause = refusal
        .coupled_producer_denial()
        .expect("the real required producer refused");
    assert_eq!(
        cause.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied,
        "the actual producer-domain refusal must be retained: {cause:?}"
    );
    assert_eq!(
        cause.subject(),
        "worth.query.certification.planar-initial.v1"
    );
    assert_eq!(
        cause.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Terminal
    );
    assert_eq!(
        cause.domain_reason(),
        Some("A current planar output is required before this decision.")
    );
}
