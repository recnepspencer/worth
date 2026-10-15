//! Independent native-owner journey; declared current delivery waits for the reviewed guard.
use super::*;
use crate::application_program::discovered_contracts::{Action, Program, QueryDelivery};
use worth_query_decl::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationIntent},
    application_program::ApplicationProgramAuthoring,
    application_schema::{ApplicationEncodedScalarValue, TypedMutationPreconditions},
};
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance as Advance, WorthQueryOperationAdvanceDenial as Denial,
    WorthQueryOperationProgress as Progress,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts, BodyKey,
    PlanarMutationScope, PlanarSourceAdjustmentBinding,
};
mod retention;

// The initiating intent cannot be cloned or reconstructed by the ordinary owner.
struct Original(PlanarSourceAdjustment);
impl ApplicationMutationIntent<ConsumerSchema> for Original {
    type Binding = PlanarSourceAdjustmentBinding<ConsumerSchema>;
    fn input(&self) -> &PlanarSourceAdjustment {
        &self.0
    }
    fn scope_binding(&self) -> PlanarMutationScope<ConsumerSchema> {
        <PlanarSourceAdjustment as ApplicationMutationIntent<ConsumerSchema>>::scope_binding(
            &self.0,
        )
    }
}

#[test]
fn discovered_owning_original_partial_recovers_native_settlement_and_closes() {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, Program<QueryDelivery>>::begin()
        .validated_program()
        .unwrap();
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<Program<QueryDelivery>>(
            "model-owner",
            "model-owner",
        ),
    ];
    let world = installation::install_selected_program(
        &crate::installed_schema(),
        8192,
        4096,
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &grants,
        program,
    )
    .unwrap();
    let adapter = authentication::admit(world.application.installed_schema());
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let operation = {
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let source = request
            .query(PlanarRead {
                body_key: input.scope_key.clone(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone();
        request
            .mutate(Original(input.clone()))
            .expect_source(source)
            .preconditions(TypedMutationPreconditions::new().expect_fact(
                BodyKey::reference::<ConsumerSchema>(),
                ApplicationEncodedScalarValue::try_new("anchor-a".to_owned()).unwrap(),
            ))
            .idempotency_owned(110_001)
            .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
            .unwrap_or_else(|(_, denial)| panic!("exact discovered construction: {denial:?}"))
    };
    // No initiating request, principal or scope remains live.
    world
        .application
        .with_host_advancement(|phase| {
            world.application.fail_next_durable_append_for_test(
                &phase
                    .checked_execution_request_for(world.application.product_runtime())
                    .unwrap(),
            )
        })
        .unwrap();
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let operation = match operation.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceUnpublished,
        } => operation,
        _ => panic!("the genuine append fault must return original partial custody"),
    };
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    assert!(operation.handler_result().is_none());
    assert!(operation.recovered_progress().is_none());
    let foreign = installation::install(&crate::installed_schema());
    let foreign_adapter = authentication::admit(foreign.application.installed_schema());
    let foreign_principal = authentication::block_on(foreign_adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let operation =
        match operation.advance(&foreign.application.request(&foreign_principal, &scope)) {
            Advance::Refused {
                operation,
                denial: Denial::FreshRequestMismatch,
            } => operation,
            _ => panic!("foreign runtime cannot consume original source custody"),
        };
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let fresh = world.application.request(&fresh_principal, &fresh_scope);
    let mut current = Some(operation);
    let settled = (0..32)
        .find_map(|_| match current.take().unwrap().advance(&fresh) {
            Advance::Progress { operation, .. } => {
                current = Some(operation);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("native discovered continuation: {denial:?}")
            }
        })
        .expect("the real finite fixture graph settles");
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    assert!(
        settled.handler_result().is_none(),
        "recovery never invents a Fresh handler result"
    );
    assert!(settled.recovered_progress().is_some());
    assert_eq!(settled.settlement().output_count(), 6);
    assert_eq!(
        fresh
            .query(PlanarRead {
                body_key: input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(2)
    );
    // This is an independent product read, not a claim that the declared aggregate is Current.
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
    let (settled, _) = settled
        .close(&world.application.request(&stopped_principal, &stopped))
        .err()
        .expect("stopped close returns the native settlement and cleanup");
    assert_eq!(settled.settlement().output_count(), 6);
    let _closed = settled
        .close(&fresh)
        .unwrap_or_else(|(_, denial)| panic!("fresh lawful cleanup: {denial:?}"));
    assert!(world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}
