//! Original move-only descriptor survives native partial publication through complete ordinary delivery.
use super::*;
use crate::application_program::discovered_delivery::{Action, Program, QueryDelivery};
use worth_query_decl::facade::{
    application_operation::ApplicationMutationIntent,
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
macro_rules! declared_world {
    ($world:ident,$request:ident,$delivery:ty) => {
        let program=ApplicationProgramAuthoring::<ConsumerSchema,Program<$delivery>>::begin().validated_program().unwrap();
        let grants=[crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<Program<$delivery>>("model-owner","model-owner")];
        let $world=installation::install_selected_program(&crate::installed_schema(),8192,4096,worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(),&grants,program).unwrap();
        let adapter=authentication::admit($world.application.installed_schema());
        let scope=authentication::request_scope();
        let principal=authentication::block_on(adapter.authenticate(authentication::LocalCredential::issued_for_model_owner(),&scope)).unwrap();
        let $request=$world.application.request(&principal,&scope);
    };
}
mod affinity;
mod delivery;
mod membership;
mod replay;
mod retention;
mod unrelated;

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
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
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
    let mut held_partial = None;
    let mut current = Some(operation);
    let settled = (0..32)
        .find_map(|_| match current.take().unwrap().advance(&fresh) {
            Advance::Progress { operation, progress } => {
                if matches!(progress,Progress::Recovery(worth_query_host::facade::application_entry::WorthQueryProgramSourceRecoveryProgress::Performed)) {
                    let page=world.application.product_publication_recovery_page(None,NonZeroUsize::new(8).unwrap()).unwrap();
                    assert_eq!(page.rows().len(),1);
                    let inspector=world.application.readmit_product_publication_recovery(page.rows()[0].handle()).unwrap();
                    held_partial=Some(inspector.inspect().unwrap());
                }
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
    let delivered = settled
        .read_current(&fresh)
        .expect("whole aggregate guard and declared selector share one phase");
    assert_eq!(delivered.rows()[0].value, length(42));
    assert_eq!(
        settled.read_historical(&fresh).unwrap().rows()[0].value,
        length(42)
    );
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
    assert!(
        held_partial.is_some(),
        "actual post-recovery caller capability retained"
    );
    let (settled, denial) = settled
        .close(&fresh)
        .err()
        .expect("native cleanup refuses the live inspector capability");
    assert!(matches!(denial,worth_query_host::facade::application_entry::WorthQueryOperationCloseDenial::Recovery(worth_query_host::facade::product::WorthQueryProductUnpublishedRecoveryReleaseDenial::World(worth_query_host::facade::primary_graph::RuntimeWorldRecoveryDenial::CallerCapabilityLive))));
    assert_eq!(
        world
            .application
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .len(),
        1
    );
    assert_eq!(settled.settlement().output_count(), 6);
    drop(held_partial);
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

#[test]
fn discovered_ordinary_executable_entry_examples_use_installed_delivery_contracts() {
    let installed = crate::installed_schema();
    super::discovered::performed_source_discovers_required_root(&installed);
    super::discovered::isolated_source_settles_without_roots(&installed);
}
