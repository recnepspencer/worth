//! Defensive ordinary handling of actual exhausted native delivery, not natural API loss.
use super::*;
use crate::application_program::ConsumerFixedAction;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequest, WorthQueryFixedOperation, WorthQueryOperationAdvance as Advance,
    WorthQueryOperationAdvanceDenial as Denial, WorthQueryOperationCloseDenial,
    WorthQueryOperationProgress as Progress,
    WorthQueryProgramSourceRecoveryProgress as RecoveryProgress, WorthQuerySettledFixedOperation,
};
use worth_query_host::facade::primary_graph::RuntimeWorldRecoveryDenial;
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
};

mod guard_lifecycle;
type Fixed<'a> = WorthQueryFixedOperation<
    'a,
    ConsumerSchema,
    PlanarSourceAdjustment,
    crate::ConsumerProgram,
    crate::ConsumerProgramRoot,
    ConsumerFixedAction,
>;
type Settled<'a> = WorthQuerySettledFixedOperation<
    'a,
    ConsumerSchema,
    PlanarSourceAdjustment,
    crate::ConsumerProgram,
    crate::ConsumerProgramRoot,
    ConsumerFixedAction,
>;
type World = crate::application_invariant_acceptance::installation::ConsumerWorld;

fn input(y: u64) -> PlanarSourceAdjustment {
    PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(y),
    }
}
fn append_failure(world: &World) {
    world
        .application
        .with_host_advancement(|phase| {
            world.application.fail_next_durable_append_for_test(
                &phase
                    .checked_execution_request_for(world.application.product_runtime())
                    .unwrap(),
            );
        })
        .unwrap();
}
fn partial<'a>(
    world: &'a World,
    request: &WorthQueryApplicationRequest<'_, '_, '_, ConsumerSchema>,
    intent: PlanarSourceAdjustment,
    key: u64,
) -> Fixed<'a> {
    let source = request
        .query(PlanarRead {
            body_key: intent.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let operation = request
        .mutate(intent)
        .expect_source(source)
        .idempotency_owned(key)
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("original description: {cause:?}"));
    append_failure(world);
    match operation.advance(request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceUnpublished,
        } => operation,
        _ => panic!("genuine append refusal retains original native partial"),
    }
}
fn recover<'a>(
    operation: Fixed<'a>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, ConsumerSchema>,
) -> Fixed<'a> {
    match operation.advance(request) {
        Advance::Progress {
            operation,
            progress: Progress::Recovery(RecoveryProgress::Performed),
        } => operation,
        Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
            panic!("native recovery: {denial:?}")
        }
        _ => panic!("actual native recovery performs once"),
    }
}
fn settle<'a>(
    operation: Fixed<'a>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, ConsumerSchema>,
) -> Settled<'a> {
    let mut operation = Some(operation);
    (0..32)
        .find_map(|_| match operation.take().unwrap().advance(request) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("valid first carrier: {denial:?}")
            }
        })
        .expect("authored finite output graph settles")
}

#[test]
fn fixed_missing_carrier_valid_public_twin() {
    let world = installation::install(&crate::installed_schema());
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let intent = input(2);
    reset_planar_source_adjustment_contacts(&intent);
    let operation = recover(partial(&world, &request, intent.clone(), 10_171), &request);
    let settled = settle(operation, &request);
    assert!(settled.handler_result().is_none());
    assert_eq!(settled.settlement().output_count(), 4);
    assert_eq!(
        settled.settlement().root_producer_contacts_in_this_demand(),
        1
    );
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(3)
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("actual cleanup: {cause:?}"));
    assert_eq!(planar_source_adjustment_contacts(&intent), (1, 1));
}

#[test]
fn fixed_missing_carrier_consumed_native_delivery_is_terminal_with_cleanup() {
    let world = installation::install(&crate::installed_schema());
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let intent = input(2);
    reset_planar_source_adjustment_contacts(&intent);
    let operation = partial(&world, &request, intent.clone(), 10_172);
    let guard = world
        .application
        .runtime()
        .consume_next_recovered_program_source_delivery_for_test()
        .unwrap();
    let operation = recover(operation, &request);
    assert!(guard
        .captured_delivery_for_test()
        .unwrap()
        .has_original_delivery());
    let page = world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let inspector = world
        .application
        .readmit_product_publication_recovery(page.rows()[0].handle())
        .unwrap();
    let held = inspector.inspect().unwrap();
    let terminal = match operation.advance(&request) {
        Advance::Terminal {
            operation,
            denial: Denial::RecoveryCarrierUnavailable,
        } => operation,
        _ => panic!("genuinely exhausted original parts terminate without output settlement"),
    };
    assert!(terminal.recovered_progress().is_some());
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: intent.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(2)
    );
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
    let (terminal, cause) = terminal
        .close(&request)
        .err()
        .expect("actual caller capability refuses cleanup");
    assert!(matches!(cause, WorthQueryOperationCloseDenial::Recovery(worth_query_host::facade::product::WorthQueryProductUnpublishedRecoveryReleaseDenial::World(RuntimeWorldRecoveryDenial::CallerCapabilityLive))));
    assert!(terminal.recovered_progress().is_some());
    assert_eq!(planar_source_adjustment_contacts(&intent), (1, 1));
    drop(held);
    drop(inspector);
    drop(guard);
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    terminal
        .close(&world.application.request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, cause)| panic!("fresh native cleanup: {cause:?}"));
    assert!(world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(planar_source_adjustment_contacts(&intent), (1, 1));
}

#[test]
fn fixed_missing_carrier_direct_query_promotion_retains_real_refusal() {
    use worth_query_host::facade::application_entry::{
        WorthQueryApplicationRecoveryRequestDenial, WorthQueryRequiredOutputRecoveryPosture,
    };
    use worth_query_host::facade::primary_graph::{
        WorthQueryManagedApplicationRecoveryDenial, WorthQueryOutputDemandDenialKind,
    };
    for consumed in [false, true] {
        let world = installation::install(&crate::installed_schema());
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let intent = input(2);
        reset_planar_source_adjustment_contacts(&intent);
        let original = request
            .query(PlanarRead {
                body_key: intent.scope_key.clone(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone();
        append_failure(&world);
        let outcome = request
            .mutate(intent.clone())
            .expect_source(original.clone())
            .idempotency(&10_173)
            .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
                &world.application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(mut recovery) =
            outcome
        else {
            panic!("real partial owns original source")
        };
        let guard = consumed.then(|| {
            world
                .application
                .runtime()
                .consume_next_recovered_program_source_delivery_for_test()
                .unwrap()
        });
        request
            .mutate(intent.clone())
            .expect_source(original.clone())
            .idempotency(&10_173)
            .recover_unpublished_required_in_program(&mut recovery, &world.application)
            .unwrap();
        if let Some(guard) = &guard {
            assert!(guard
                .captured_delivery_for_test()
                .unwrap()
                .has_original_delivery());
        }
        let promotion = request
            .mutate(intent.clone())
            .expect_source(original)
            .idempotency(&10_173)
            .promote_recovered_required_outputs(recovery, &world.application);
        if consumed {
            let (cause, recovery) = promotion
                .err()
                .expect("second native take cannot promote missing parts");
            assert!(
                matches!(cause, WorthQueryApplicationRecoveryRequestDenial::Recovery(WorthQueryManagedApplicationRecoveryDenial::Demand(ref native)) if native.kind() == WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable)
            );
            assert_eq!(
                recovery.recovery_posture(),
                WorthQueryRequiredOutputRecoveryPosture::Terminal
            );
            assert!(recovery.performed().is_some());
            drop(recovery);
        } else {
            let (preparation, _, performed, failures) = promotion
                .unwrap_or_else(|(cause, _)| panic!("first carrier genuinely promotes: {cause:?}"))
                .into_parts();
            assert!(failures.is_empty());
            let mut outputs = preparation
                .start_required_outputs(
                    &world.application,
                    &request,
                    WorthQueryOutputDemandControls::new(
                        NonZeroUsize::new(4096).unwrap(),
                        NonZeroUsize::new(8192).unwrap(),
                    ),
                )
                .unwrap_or_else(|(_, cause)| panic!("valid output owner: {cause:?}"));
            super::super::settle(|| {
                match outputs.advance(&world.application, &request).unwrap() {
                worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputProgress::Pending => None,
                worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
            }
            });
            drop(outputs);
            drop(performed);
        }
        drop(guard);
        let page = world
            .application
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert_eq!(page.rows().len(), 1);
        let recovery = world
            .application
            .readmit_product_publication_recovery(page.rows()[0].handle())
            .unwrap();
        world
            .application
            .release_product_publication_recovery(recovery, 0)
            .unwrap();
        assert_eq!(planar_source_adjustment_contacts(&intent), (1, 1));
    }
}
