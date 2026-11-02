use super::*;
use std::sync::atomic::Ordering;

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
    assert_domain_denial(edit.terminal_source().unwrap().advance_denial());
    assert!(
        matches!(
            edit.terminal_source().unwrap().advance_denial(),
            Some(WorthQueryOperationAdvanceDenial::Outputs(_))
        ),
        "the original typed output refusal remains on the same terminal owner"
    );
    let cancellation=worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(),
        );
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
        WorthQueryHistoryEditDenial::Close(WorthQueryOperationCloseDenial::Execution(_))
    ));
    assert!(matches!(
        edit.terminal_source().unwrap().advance_denial(),
        Some(WorthQueryOperationAdvanceDenial::Outputs(_))
    ));
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
    assert_domain_denial(terminal.advance_denial());
    assert!(
        matches!(
            terminal.advance_denial(),
            Some(WorthQueryOperationAdvanceDenial::Outputs(_))
        ),
        "native close preserves the original typed output refusal"
    );
    assert!(
        terminal.source_outcome().is_none(),
        "the source performed before producer denial"
    );
    assert!(
        terminal.source_result().is_some(),
        "the original performed source result survives native close"
    );
    assert!(
        terminal.close_evidence().output_unavailable().is_some(),
        "original saved-source/unavailable proof remains distinct from no effect"
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
    drop(terminal);
    application
        .with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

fn assert_domain_denial(denial: Option<&WorthQueryOperationAdvanceDenial>) {
    let Some(WorthQueryOperationAdvanceDenial::Outputs(
        WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::Demand(cause),
        ),
    )) = denial
    else {
        panic!("actual producer domain refusal required: {denial:?}")
    };
    assert_eq!(cause.kind(),worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProducerDomainDenied);
    assert_eq!(
        cause.domain_reason(),
        Some("A current planar output is required before this decision.")
    );
}
