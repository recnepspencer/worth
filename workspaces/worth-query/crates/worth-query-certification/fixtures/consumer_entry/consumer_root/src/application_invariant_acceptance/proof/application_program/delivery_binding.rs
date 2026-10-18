//! A public selector lookalike cannot replace the installed concrete delivery.
use super::*;
use crate::application_program::{ConsumerFixedAction, ConsumerFixedDelivery, ConsumerProgramRoot};
use worth_query_decl::facade::application_program::{
    ApplicationActionDeliveryDeclaration, ApplicationActionReference,
    ApplicationFixedActionDelivery, ApplicationFixedQueryDelivery, ApplicationRootComposition,
};
use worth_query_host::facade::application_entry::WorthQueryOperationConstructionDenial;
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts, PlanarOutputRead,
    PlanarOutputReadBinding, PlanarSourceAdjustmentBinding, PlanarSourceFeature,
};

pub struct CounterfeitDelivery;
impl ApplicationFixedActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for CounterfeitDelivery
{
    type Root = ConsumerProgramRoot;
    type Projection = worth_query_decl::facade::application_program::ApplicationQueryProjection<
        PlanarOutputReadBinding<ConsumerSchema>,
    >;
}
impl
    worth_query_decl::facade::application_program::ApplicationFixedQueryDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    > for CounterfeitDelivery
{
    type Read = PlanarOutputReadBinding<ConsumerSchema>;

    fn read_input(_: &PlanarSourceAdjustment) -> PlanarOutputRead {
        PlanarOutputRead {
            body_key: "different-selector".to_owned(),
        }
    }
}
type CounterfeitAction = ApplicationActionReference<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    CounterfeitDelivery,
>;

#[test]
fn counterfeit_delivery_selector_refuses_and_installed_twin_constructs() {
    let world = installation::install(&crate::installed_schema());
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    assert_ne!(
        CounterfeitDelivery::read_input(&input).body_key,
        ConsumerFixedDelivery::read_input(&input).body_key
    );
    let counterfeit = ApplicationActionDeliveryDeclaration::fixed::<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
        CounterfeitDelivery,
    >();
    let installed = ApplicationActionDeliveryDeclaration::fixed::<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
        ConsumerFixedDelivery,
    >();
    assert_eq!(
        counterfeit.canonical_meaning(),
        installed.canonical_meaning(),
        "runtime implementation identity is not serialized"
    );
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let description = request
        .mutate(input.clone())
        .expect_source(source)
        .idempotency_owned(10_063)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"));
    let (description, denial) = match description
        .retain_operation::<crate::ConsumerProgram, CounterfeitAction>(&world.application)
    {
        Err(refusal) => refusal,
        Ok(_) => panic!("counterfeit concrete selector was accepted by the real constructor"),
    };
    assert!(matches!(
        denial,
        WorthQueryOperationConstructionDenial::Action(
            worth_query_host::facade::domain::WorthQueryActionResolutionDenial::DeliveryMismatch
        )
    ));
    assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
    let _valid = description
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, denial)| {
            panic!("returned original description admits the installed twin: {denial:?}")
        });
    assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
}

impl
    worth_query_decl::facade::application_program::ApplicationActionDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    > for CounterfeitDelivery
{
    type Mode = worth_query_decl::facade::application_program::ApplicationFixedDeliveryMode<
        ConsumerProgramRoot,
        worth_query_decl::facade::application_program::ApplicationQueryProjection<
            PlanarOutputReadBinding<ConsumerSchema>,
        >,
    >;
    const IDENTITY: &'static str = <ConsumerFixedDelivery as worth_query_decl::facade::application_program::ApplicationActionDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    >>::IDENTITY;
}

pub struct SourceRequestedForRequired;
impl
    worth_query_decl::facade::application_program::ApplicationActionDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    > for SourceRequestedForRequired
{
    type Mode = worth_query_decl::facade::application_program::ApplicationSourceCompletion;
    const IDENTITY: &'static str = <ConsumerFixedDelivery as worth_query_decl::facade::application_program::ApplicationActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>>::IDENTITY;
}
type SourceRequestedAction = ApplicationActionReference<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    SourceRequestedForRequired,
>;
#[test]
fn common_operation_handoff_required_output_cannot_select_source_completion() {
    let world = installation::install(&crate::installed_schema());
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let description = request
        .mutate(input.clone())
        .expect_source(source)
        .idempotency_owned(10_064)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"));
    let (description, denial) = match description
        .retain_operation::<crate::ConsumerProgram, SourceRequestedAction>(&world.application)
    {
        Err(refusal) => refusal,
        Ok(_) => panic!("required output cannot be replaced by source completion"),
    };
    assert!(matches!(
        denial,
        WorthQueryOperationConstructionDenial::Action(
            worth_query_host::facade::domain::WorthQueryActionResolutionDenial::DeliveryMismatch
        )
    ));
    assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
    let _fixed = description
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, denial)| {
            panic!("returned original description preserves fixed delivery: {denial:?}")
        });
}
