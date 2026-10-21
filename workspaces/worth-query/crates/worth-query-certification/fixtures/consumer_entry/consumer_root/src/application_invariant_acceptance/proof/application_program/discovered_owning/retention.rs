//! Ordinary committed-refusal custody retries the actual original carrier without another effect.
use super::*;

#[test]
fn discovered_owning_committed_retention_retry_preserves_fresh_result_and_source() {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, Program<QueryDelivery>>::begin()
        .validated_program()
        .unwrap();
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<Program<QueryDelivery>>(
            "model-owner",
            "model-owner",
        ),
    ];
    let mut world = installation::install_selected_program(
        &crate::installed_schema(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &grants,
        program,
    )
    .unwrap();
    let request_runtime = world.application.clone();
    let constructing = std::mem::replace(&mut world.application, request_runtime);
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let prior_source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    // A genuine earlier ordinary owner registers, then abandons its unstarted outputs.
    let prior = request
        .mutate(Original(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        }))
        .expect_source(prior_source)
        .idempotency_owned(110_011)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
        .unwrap_or_else(|(_, denial)| panic!("prior ordinary construction: {denial:?}"));
    let prior = match prior.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        _ => panic!("real prior must commit and register"),
    };
    drop(prior);
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(2),
        "the prior effect really precedes the new 2-to-1 transition"
    );
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(1),
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
    let operation = {
        let construction_scope = authentication::request_scope();
        let construction_principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &construction_scope,
        ))
        .unwrap();
        let construction_request =
            constructing.request(&construction_principal, &construction_scope);
        construction_request
            .mutate(Original(input.clone()))
            .expect_source(source)
            .preconditions(TypedMutationPreconditions::new().expect_fact(
                BodyKey::reference::<ConsumerSchema>(),
                ApplicationEncodedScalarValue::try_new("anchor-a".to_owned()).unwrap(),
            ))
            .idempotency_owned(110_012)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&constructing)
            .unwrap_or_else(|(_, denial)| panic!("ordinary installed construction: {denial:?}"))
    };
    let guard = world
        .application
        .refuse_next_discovered_retention_reservation_for_test()
        .unwrap();
    let operation = match operation.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceRetentionDenied,
        } => operation,
        _ => panic!("actual reservation must refuse after committing the source"),
    };
    drop(guard);
    drop(constructing);
    assert_eq!(operation.handler_result().unwrap().changed_vertices, 1);
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(1)
    );
    let original_commit = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(),
        );
    let stopped_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &stopped,
    ))
    .unwrap();
    cancellation.cancel();
    let operation =
        match operation.advance(&world.application.request(&stopped_principal, &stopped)) {
            Advance::Refused {
                operation,
                denial: Denial::Execution(_),
            } => operation,
            _ => panic!("fresh stopped request cannot consume the committed carrier"),
        };
    assert_eq!(operation.handler_result().unwrap().changed_vertices, 1);
    let operation = match operation.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        _ => panic!("fresh authorized retry promotes the same original source"),
    };
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &original_commit
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    let mut current = Some(operation);
    let settled = (0..32)
        .find_map(|_| match current.take().unwrap().advance(&request) {
            Advance::Progress { operation, .. } => {
                current = Some(operation);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("native fanout: {denial:?}")
            }
        })
        .expect("the real bounded graph settles");
    assert_eq!(settled.settlement().output_count(), 6);
    assert_eq!(settled.handler_result().unwrap().changed_vertices, 1);
    assert!(
        settled.recovered_progress().is_none(),
        "native retention retry preserves Fresh result, not recovered projection"
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    use worth_query_host::facade::application_entry::{
        WorthQueryOperationDeliveryDenial, WorthQueryOperationDeliveryPolicy as Policy,
    };
    let one = NonZeroUsize::new(1).unwrap();

    let policy = Policy::default();
    let query_denial = settled
        .read_current_with_policy(&request, policy.query(NonZeroUsize::new(2).unwrap()))
        .err()
        .expect("declared query must refuse widening its semantic result cardinality");
    assert!(
        matches!(
            &query_denial,
            WorthQueryOperationDeliveryDenial::Query(worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::Limit(_))
        ),
        "actual declared query refusal: {query_denial:?}"
    );
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let fresh = world.application.request(&fresh_principal, &fresh_scope);
    let delivered = settled
        .read_current(&fresh)
        .expect("same owner retries declared delivery with fresh authorization");
    assert_eq!(delivered.rows()[0].value, length(42));
    assert_eq!(
        settled.read_historical(&fresh).unwrap().rows()[0].value,
        length(42)
    );
    assert_eq!(
        fresh.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    let _closed = settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("lawful native cleanup: {denial:?}"));
}
