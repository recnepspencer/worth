//! A newly performed source can be fulfilled by genuinely restored output authority.
use super::*;
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::application_entry::{
    WorthQueryFixedOperation, WorthQueryOutputSettlementPosture, WorthQuerySettledFixedOperation,
};
use worth_query_topology_entry::{PlanarAnchorOutput, PlanarOutputs};
type Operation = WorthQueryFixedOperation<
    ConsumerSchema,
    PlanarSourceAdjustment,
    crate::ConsumerProgram,
    crate::ConsumerProgramRoot,
    ConsumerFixedAction,
>;
type Settled = WorthQuerySettledFixedOperation<
    ConsumerSchema,
    PlanarSourceAdjustment,
    crate::ConsumerProgram,
    crate::ConsumerProgramRoot,
    ConsumerFixedAction,
>;

fn complete(
    mut operation: Option<Operation>,
    application: &super::super::super::ProgramApplication,
) -> Settled {
    let adapter = authentication::admit(application.installed_schema());
    for _ in 0..64 {
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        match operation
            .take()
            .unwrap()
            .advance(&application.request(&principal, &scope))
        {
            Advance::Progress {
                operation: next, ..
            } => operation = Some(next),
            Advance::Settled(settled) => return settled,
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("authentic restored source progression: {denial:?}")
            }
        }
    }
    panic!("bounded real source and output traversal did not settle");
}

#[test]
fn a_new_performed_source_is_fulfilled_by_restored_outputs_without_reexecution() {
    let world = installation::install(&crate::installed_schema());
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let original = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let operation = request
        .mutate(input.clone())
        .expect_source(original)
        .idempotency_owned(0x9182_c100)
        .unwrap_or_else(|(_, _, cause)| panic!("capture: {cause:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("retain: {cause:?}"));
    let settled = complete(Some(operation), &world.application);
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(3)
    );
    let anchor = settled
        .settlement()
        .root_settlement()
        .outputs_of::<PlanarOutputs>()
        .unwrap()
        .entity::<PlanarAnchorOutput<ConsumerSchema>>()
        .unwrap()
        .entity_id();
    settled
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("original close: {cause:?}"));
    let capture = crate::application_invariant_acceptance::resources::checkpoint_capture_request();
    let checkpoint = world.application.with_funded_host_advancement(
        worth_query_host::facade::runtime::ExecutionAllocationPolicy::Execution(&capture),
        |phase| world.application.runtime()
            .capture_application_checkpoint(&phase, worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy::SystemAllocation).unwrap()
    ).unwrap();
    let program = ApplicationProgramAuthoring::<ConsumerSchema, crate::ConsumerProgram>::begin()
        .with_schema(&<ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap())
        .validated_program()
        .unwrap();
    let restored = installation::restore_selected_program(&world, program, checkpoint);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(restored.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = restored.request(&principal, &scope);
    let current = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let operation = request
        .mutate(input.clone())
        .expect_source(current)
        .idempotency_owned(0x9182_c101)
        .unwrap_or_else(|(_, _, cause)| panic!("restored capture: {cause:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&restored)
        .unwrap_or_else(|(_, cause)| panic!("restored retain: {cause:?}"));
    let operation = match operation.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        _ => panic!("the new key performs the real equal-value source action"),
    };
    let source_receipt = operation.prepared_source_receipt().unwrap().clone();
    let settled = complete(Some(operation), &restored);
    let source = settled.settlement().performed_source().unwrap();
    assert!(source
        .receipt()
        .is_same_authoritative_commit(&source_receipt));
    assert!(source.root_fulfillment().matches_source(&source_receipt));
    assert_eq!(
        source.root_fulfillment().output_commit(),
        settled.settlement().root_observation().selected_commit()
    );
    assert_eq!(
        source.root_fulfillment().posture(),
        settled.settlement().root_settlement().posture()
    );
    assert!(matches!(
        source.root_fulfillment().posture(),
        WorthQueryOutputSettlementPosture::RecoveredPerformed
            | WorthQueryOutputSettlementPosture::RecoveredStableReused
    ));
    assert_eq!(
        settled.settlement().root_producer_contacts_in_this_demand(),
        0
    );
    assert_eq!(
        settled
            .settlement()
            .root_settlement()
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<ConsumerSchema>>()
            .unwrap()
            .entity_id(),
        anchor
    );
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(3)
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("restored close: {cause:?}"));
}
