//! Public owning construction and delivery use the exact canonical action instance.
use super::*;
use crate::application_program::{
    action_instances::*, ConsumerFixedAction, ConsumerFixedDelivery, SecondaryPlanarRoot,
};
use worth_query_decl::facade::application_program::{
    ApplicationAction, ApplicationActionDelivery, ApplicationCompositionInstance,
    ApplicationProgramAuthoring,
};
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance as Advance, WorthQueryOperationConstructionDenial,
};
use worth_query_host::facade::domain::WorthQueryActionResolutionDenial;
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
    PlanarSourceAdjustmentBinding,
};

macro_rules! settle {
    ($operation:expr, $request:expr) => {{
        let mut operation = Some($operation);
        (0..32)
            .find_map(|_| match operation.take().unwrap().advance($request) {
                Advance::Progress {
                    operation: next, ..
                } => {
                    operation = Some(next);
                    None
                }
                Advance::Settled(settled) => Some(settled),
                Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                    panic!("exact instance progression: {denial:?}")
                }
            })
            .expect("real installed root settles")
    }};
}

#[test]
fn fixed_action_instances_deliver_only_their_canonical_contract() {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, ActionInstanceProgram>::begin()
        .validated_program()
        .unwrap();
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<ActionInstanceProgram>(
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
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    type Binding = PlanarSourceAdjustmentBinding<ConsumerSchema>;
    let primary_declaration =
        <ConsumerFixedAction as ApplicationAction<ConsumerSchema, Binding>>::declaration();
    let secondary_declaration =
        <SecondaryAction as ApplicationAction<ConsumerSchema, Binding>>::declaration();
    assert_eq!(
        primary_declaration.binding(),
        secondary_declaration.binding()
    );
    assert_ne!(
        primary_declaration.composition_instance(),
        secondary_declaration.composition_instance()
    );
    assert_ne!(
        primary_declaration.delivery(),
        secondary_declaration.delivery()
    );

    let secondary_input = PlanarSourceAdjustment {
        scope_key: "anchor-b".to_owned(),
        replacement_y: length(5),
    };
    reset_planar_source_adjustment_contacts(&secondary_input);
    let source = request
        .query(PlanarRead {
            body_key: secondary_input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let description = request
        .mutate(secondary_input.clone())
        .expect_source(source)
        .idempotency_owned(10_192)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"));
    let before = world
        .application
        .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
        .unwrap();
    let (description, denial) = match description
        .retain_operation::<ActionInstanceProgram, WrongSecondaryAction>(&world.application)
    {
        Err(refusal) => refusal,
        Ok(_) => panic!("primary delivery cannot substitute at the secondary occurrence"),
    };
    assert!(matches!(
        denial,
        WorthQueryOperationConstructionDenial::Action(
            WorthQueryActionResolutionDenial::DeliveryMismatch
        )
    ));
    assert_eq!(planar_source_adjustment_contacts(&secondary_input), (0, 0));
    let after = world
        .application
        .with_host_advancement(|phase| request.retain_read(&phase).unwrap())
        .unwrap();
    assert_eq!(after.selected_commit(), before.selected_commit());
    let secondary = description
        .retain_operation::<ActionInstanceProgram, SecondaryAction>(&world.application)
        .unwrap_or_else(|(_, denial)| {
            panic!("returned original description selects actual secondary contract: {denial:?}")
        });
    let secondary = settle!(secondary, &request);
    assert_eq!(secondary.declared_action(), &secondary_declaration);
    assert_eq!(
        secondary.declared_action().composition_instance(),
        SecondaryPlanarRoot::PATH
    );
    assert_eq!(
        secondary.declared_action().delivery().unwrap().identity(),
        <SecondarySourceDelivery as ApplicationActionDelivery<ConsumerSchema, Binding>>::IDENTITY
    );
    assert_eq!(
        secondary.settlement().output_count(),
        0,
        "the secondary graph has no dependent outputs"
    );
    assert_eq!(
        secondary
            .settlement()
            .root_producer_contacts_in_this_demand(),
        1,
        "the actual root producer still ran"
    );
    let source_delivery = secondary.read_current(&request).unwrap();
    assert_eq!(source_delivery.rows()[0].body_key, "anchor-b");
    assert_eq!(source_delivery.rows()[0].y, length(5));
    assert_eq!(planar_source_adjustment_contacts(&secondary_input), (1, 1));
    secondary
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("secondary close: {cause:?}"));

    // compiled-example: begin
    let primary_input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&primary_input);
    let source = request
        .query(PlanarRead {
            body_key: primary_input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let primary = request
        .mutate(primary_input.clone())
        .expect_source(source)
        .idempotency_owned(10_193)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<ActionInstanceProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, denial)| panic!("primary exact contract: {denial:?}"));
    let mut primary = Some(primary);
    let primary = (0..32)
        .find_map(|_| match primary.take().unwrap().advance(&request) {
            Advance::Progress { operation, .. } => {
                primary = Some(operation);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("native fixed advancement: {denial:?}")
            }
        })
        .expect("the declared fixed graph settles");
    assert_eq!(primary.declared_action(), &primary_declaration);
    assert_eq!(
        primary.declared_action().delivery().unwrap().identity(),
        <ConsumerFixedDelivery as ApplicationActionDelivery<ConsumerSchema, Binding>>::IDENTITY
    );
    assert_eq!(primary.settlement().output_count(), 4);
    let output_delivery = primary.read_current(&request).unwrap();
    assert_eq!(output_delivery.rows()[0].body_key, "anchor-a");
    assert_eq!(output_delivery.rows()[0].value, length(3));
    assert_eq!(planar_source_adjustment_contacts(&primary_input), (1, 1));
    primary
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("primary close: {cause:?}"));
    // compiled-example: end
}
