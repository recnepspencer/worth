use super::*;
#[test]
fn fixed_superseded_registration_is_terminal_historical_evidence() {
    let foreign = crate::installed_schema();
    let world = installation::install(&foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-b".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let first = request
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency_owned(10_091)
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("construction: {cause:?}"));
    let first = match first.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        _ => panic!("first source publication performs"),
    };
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    drop(first);
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1
    );
    let current = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let newer = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".to_owned(),
            replacement_y: length(6),
        })
        .expect_source(current)
        .idempotency_owned(10_092)
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("successor construction: {cause:?}"));
    let newer = match newer.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        _ => panic!("actual newer source publication supersedes the old registration"),
    };
    drop(newer);
    let replay = request
        .mutate(input.clone())
        .expect_source(source)
        .idempotency_owned(10_091)
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("historical construction: {cause:?}"));
    let replay = match replay.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceReplayObserved,
        } => operation,
        _ => panic!("original committed source evidence remains available"),
    };
    let terminal = match replay.advance(&request) {
        Advance::Terminal { operation, denial: worth_query_host::facade::application_entry::WorthQueryOperationAdvanceDenial::Outputs(
            worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
                worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(cause))) } => {
            assert_eq!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable);
            operation
        },
        Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => panic!("wrong supersession refusal: {denial:?}"),
        _ => panic!("superseded registration cannot settle its outputs"),
    };
    assert!(matches!(terminal.native_outcome(), Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(_))));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-b".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(6)
    );
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("historical cleanup: {cause:?}"));
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
}
