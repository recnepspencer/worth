//! No-output construction checks the native compiled inventory, not an author flag.
use super::*;
use worth_query_decl::facade::application_program::*;
use worth_query_topology_entry::{PlanarSourceAdjustmentBinding, PlanarSourceFeature};
struct Completion;
impl ApplicationActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for Completion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "required-source-requested-as-completion.v1";
}
struct RequiredSourceRequestedAsCompletion;
impl ApplicationProgramDefinition<ConsumerSchema> for RequiredSourceRequestedAsCompletion {
    type Contributions =
        <crate::ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs =
        <crate::ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = <crate::ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("native-output-source-inventory-negative");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        crate::application_program::source_delivery_specs::<Completion>()
    }
}
type RequestedAction = ApplicationActionReference<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    Completion,
>;
#[test]
fn common_operation_handoff_native_inventory_refuses_required_source() {
    let world = installation::install_selected_program(
        &crate::installed_schema(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &[],
        ApplicationProgramAuthoring::<ConsumerSchema, RequiredSourceRequestedAsCompletion>::begin()
            .validated_program()
            .unwrap(),
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
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let description = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency_owned(10_065)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"));
    let (_, denial) = match description
        .retain_operation::<RequiredSourceRequestedAsCompletion, RequestedAction>(
            &world.application,
        ) {
        Err(refusal) => refusal,
        Ok(_) => panic!("a no-output author flag cannot erase native required output ownership"),
    };
    assert!(matches!(denial, worth_query_host::facade::application_entry::WorthQueryOperationConstructionDenial::RequiredOutputSource));
}
