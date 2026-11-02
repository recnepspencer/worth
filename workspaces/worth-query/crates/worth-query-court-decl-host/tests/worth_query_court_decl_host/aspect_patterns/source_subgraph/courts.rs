use super::*;
fn expected(key: &str, left: u64, right: u64) -> GraphRow {
    GraphRow {
        key: key.into(),
        value: 1,
        members: vec![
            MemberRow {
                key: "left".into(),
                value: left,
            },
            MemberRow {
                key: "right".into(),
                value: right,
            },
        ],
    }
}
#[test]
fn partial_member_edit_pinned_previous_graph_and_recorded_undo_preserve_neighbors() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = request.retain_current_read().unwrap();
    assert_eq!(world::source(&request), expected("source", 11, 22));
    let receipt = world::edit_source(&host, &request, 15);
    assert_eq!(world::source(&request), expected("source", 15, 22));
    let previous = request
        .at(&before)
        .query(graph_read::source::Read("source".into()))
        .execute()
        .unwrap();
    assert_eq!(previous.rows(), &[expected("source", 11, 22)]);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    let preparation = request
        .corrections()
        .reverse(&selected)
        .idempotency("undo-member")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = preparation else {
        panic!("exact retained member preimage must prepare");
    };
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    assert_eq!(world::source(&request), expected("source", 11, 22));
    // Reacquire the old graph through authentic pinned custody after undo.
    let previous_again = request
        .at(&before)
        .query(graph_read::source::Read("source".into()))
        .execute()
        .unwrap();
    assert_eq!(previous_again.rows(), &[expected("source", 11, 22)]);
}
#[test]
fn parent_witness_covers_a_generated_child_that_readiness_does_not_select() {
    let hashes = Hashes::default();
    let host = world::install(world::Fault::None, hashes.clone());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let parent = world::settle(&host, &request);
    parent.deliver_current(&request).unwrap();
    // Literal roster independently checks both actual relation targets and values.
    assert_eq!(world::generated(&request), expected("generated", 11, 22));
    assert_eq!(hashes.lock().unwrap().len(), 1);
    world::corrupt_child(&host, &request);
    assert_eq!(world::source(&request), expected("source", 11, 22));
    assert_eq!(world::generated(&request), expected("generated", 11, 999));
    let Err(error) = parent.deliver_current(&request) else {
        panic!("a child-only edit must invalidate the parent witness");
    };
    let WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(
        denial,
    )) = error
    else {
        panic!("{error:?}");
    };
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    parent
        .close(&request)
        .unwrap_or_else(|error| panic!("close: {:?}", error.denial()));
}
#[test]
fn omitting_a_required_generated_member_certificate_refuses_before_publication() {
    let host = world::install(world::Fault::OmitRightRole, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = request.retain_current_read().unwrap();
    let mut operation = request
        .demand(producer::Demand)
        .start_in_program::<program::Program, program::Graph>(&host)
        .unwrap();
    let result = operation.settle(&request);
    let Err(denial) = result else {
        panic!("Right's required role was omitted");
    };
    let WorthQueryApplicationOutputDemandDenial::Demand(denial) = denial else {
        panic!("{denial:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ProducerUnavailable
    );
    // The public envelope preserves this owner cause as diagnostic detail.
    // There is no public typed attempt-cause accessor on a demand refusal.
    assert!(denial.subject().contains("MissingOutputRole"), "{denial:?}");
    assert!(denial.subject().contains("right"), "{denial:?}");
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    assert_unpublished(&request);
    operation.close();
}

#[test]
fn native_decision_reads_refuse_a_corrupted_projected_member() {
    let host = world::install(world::Fault::CorruptInputMember, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = request.retain_current_read().unwrap();
    let mut operation = request
        .demand(producer::Demand)
        .start_in_program::<program::Program, program::Graph>(&host)
        .unwrap();
    let result = operation.settle(&request);
    let Err(denial) = result else {
        panic!("corrupted input must refuse against native facts");
    };
    let WorthQueryApplicationOutputDemandDenial::Demand(denial) = denial else {
        panic!("{denial:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert_eq!(
        denial
            .decode_producer_denial::<edit::ProducerDenialBinding>()
            .unwrap(),
        edit::Denial::SourceMismatch
    );
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    assert_unpublished(&request);
    operation.close();
}
fn assert_unpublished(request: &world::Request<'_, '_, '_>) {
    assert_eq!(world::source(request), expected("source", 11, 22));
    let result = request
        .query(graph_read::generated::Read("generated".into()))
        .execute();
    let Err(denial) = result else {
        panic!("refused candidate must not expose a generated root");
    };
    let WorthQueryApplicationRequestQueryDenial::ScopeResolution(denial) = denial else {
        panic!("{denial:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryEntityResolutionDenialKind::Refused(
            WorthQueryEntityResolutionRefusalKind::UnknownEntity
        )
    );
}
pub(crate) fn fingerprint(left: u64) -> (String, [u8; 32]) {
    let hashes = Hashes::default();
    let host = world::install(world::Fault::None, hashes.clone());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    if left != 11 {
        world::edit_source(&host, &request, left);
    }
    assert_eq!(world::source(&request), expected("source", left, 22));
    let content = fingerprint::content(&world::source(&request));
    let parent = world::settle(&host, &request);
    parent.deliver_current(&request).unwrap();
    assert_eq!(world::generated(&request), expected("generated", left, 22));
    let captured = hashes.lock().unwrap();
    assert_eq!(captured.len(), 1, "one actual SDK source-epoch handoff");
    let identity = captured[0];
    drop(captured);
    parent
        .close(&request)
        .unwrap_or_else(|error| panic!("close: {:?}", error.denial()));
    (content, identity)
}
