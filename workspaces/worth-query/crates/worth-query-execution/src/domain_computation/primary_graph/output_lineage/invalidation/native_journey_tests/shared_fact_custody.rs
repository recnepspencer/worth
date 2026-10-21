//! Independent consumed readers and restored marks share one original fact allocation.
use super::*;
use crate::domain_computation::primary_graph::{
    invariant_projection::{ConsumedOutputEvidence, ConsumedOutputVerification},
    output_lineage::RetainedSourceFacts,
    tests::application_attempt::resolved_account,
};

#[test]
fn shared_original_facts_survive_restored_prerequisite_custody() {
    let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request();
        let execution = custody.borrowed_request_for_test();
        let entity = resolved_account(&custody.test_host_request(), &world, "open", &live_scope())
            .entity_id();
        let (_, product, _) = world.selected_product(&custody.test_host_request()).into_parts();
        let coordinate = ProductCoordinate {
            occurrence: product.observation().lifecycle_incarnation(),
            generation: product.observation().reference_generation().get(),
        };
        let graph = world.application.runtime.primary_graph().unwrap();
        let handle = graph.integration_handle();
        let owner = &handle.source_owner.invalidation_owner;
        let status = AccountStatus::reference();
        let status = graph.layout().field_locator(status.entity(), status.aspect(), status.field())
            .unwrap().clone();
        let (snapshot_handle, basis, identity, _facts, first, second) = handle.with_runtime_mut(&execution, |runtime| {
            let (snapshot_handle, basis) = snapshot(runtime);
            let basis = Arc::new(basis);
            let identity = RecordedSettlementIdentity::retain(&SemanticSource {
                runtime_authority: world.application.runtime.authority_identity().as_u64(),
                schema: world.application.installed_schema.binding_identity(),
                scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
                output_binding: OutputBindingIdentity::declared("StatusOutput"),
            }, coordinate, 0);
            let facts = RetainedSourceFacts::for_test(false, Arc::from([
                field_fact(&execution, runtime, &snapshot_handle, entity, status),
            ])).for_comparison().unwrap();
            let witness = super::request_recording::account_witness(
                &execution, &world, runtime, &snapshot_handle, entity,
            );
            assert_eq!(ConsumedOutputEvidence::verify_at_observation(
                &execution, &identity, &facts, &[],
                Some(super::super::FullVerificationReason::CheckpointRestore),
                &witness, owner, runtime, &snapshot_handle, &basis,
            ).unwrap(), ConsumedOutputVerification::Current);


            let first = ConsumedOutputEvidence::restored_with_witness_for_test(Arc::clone(&identity), facts.facts().clone(), Arc::clone(&basis), Arc::clone(&witness));


            let second = ConsumedOutputEvidence::restored_with_witness_for_test(Arc::clone(&identity), facts.facts().clone(), Arc::clone(&basis), witness);

            (snapshot_handle, basis, identity, facts, first, second)
        });
        first.establish_restored_before_commit(&execution, &handle.source_owner).unwrap();
            assert!(matches!(owner.currentness(&basis, &identity).unwrap(),
                SourceSettlementCurrentness::Clean));
            assert!(first.claim_verified_checkpoint_root(owner).unwrap(),
                "actual restored prerequisite custody must be established, not bypassed");
        handle.with_runtime_mut(&execution, |runtime| {
            assert_eq!(ConsumedOutputEvidence::verify_many_with_admission(
                &execution, &[first.clone(), second.clone()], owner, runtime, &snapshot_handle,
                &basis,
            ).unwrap(), ConsumedOutputVerification::Current);
            let (current_handle, current_basis) = snapshot(runtime);
            assert_eq!(current_basis, *basis,
                "custody and currentness do not publish another native head");
            runtime.snapshots().release_snapshot(&current_handle).unwrap();
        });
            drop(first);
            drop(second);

            assert!(matches!(owner.currentness(&basis, &identity).unwrap(),
                SourceSettlementCurrentness::Clean));
            assert_eq!(owner.retire_settlements(std::slice::from_ref(&identity)),
                vec![identity]);

        handle.with_runtime_mut(&execution, |runtime| {
            runtime.snapshots().release_snapshot(&snapshot_handle).unwrap();
        });
    }).expect("the native custody journey owns its installed host call");
}
