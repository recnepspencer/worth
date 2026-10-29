pub(crate) use super::*;

#[test]
fn uninterrupted_combined_candidate_publishes_both_owner_successors() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let token = RuntimeWorldCancellationSource::new().token();
    let prepared = combined_candidate(&court, execution, &root, &token);
    let RuntimeWorldPublicationOutcome::Performed(done) =
        court.world.publication_port().execute_with_signal(
            *execution,
            prepared,
            &mut court.context(execution, &root),
            &token,
            |_| Ok(()),
        )
    else {
        panic!("the same combined candidate succeeds without cancellation");
    };
    let head = court.observe(&root);
    assert_eq!(head.selected_commit(), done.commit().identity());
    assert_ne!(
        head.basis().relational_basis().admission_identity(),
        root.basis().relational_basis().admission_identity()
    );
    assert_ne!(
        head.basis().signal_basis().admission_identity(),
        root.basis().signal_basis().admission_identity()
    );
    assert_eq!(
        court
            .records
            .read(execution, head.basis().relational_basis())
            .records
            .get("grain")
            .map(String::as_str),
        Some("5")
    );
    drop(done.consume());
    drop((head, root));
    court.finish();
}

pub(crate) fn combined_candidate(
    court: &CompositeSupplyChainCourt,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    root: &ProductBranchObservation,
    token: &RuntimeWorldCancellationToken,
) -> PreparedCompositePublicationWithSignal {
    let candidate =
        court
            .records
            .candidate(execution, root.basis().relational_basis(), "grain", "5");
    court
        .world
        .publication_port()
        .prepare_with_signal(
            root.clone(),
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(candidate),
            token,
            None,
        )
        .unwrap()
}
