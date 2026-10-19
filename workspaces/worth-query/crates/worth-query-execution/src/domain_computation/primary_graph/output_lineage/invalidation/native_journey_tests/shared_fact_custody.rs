//! Independent consumed readers and restored marks share one original fact allocation.
use super::*;
use crate::domain_computation::primary_graph::{
    invariant_projection::{ConsumedOutputEvidence, ConsumedOutputVerification},
    output_lineage::RetainedSourceFacts,
    tests::application_attempt::resolved_account,
};

#[test]
fn shared_original_facts_fund_once_and_survive_restored_prerequisite_custody() {
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
                &witness, owner, runtime, &snapshot_handle, &basis, &mut owner.edit_admission(),
            ).unwrap(), ConsumedOutputVerification::Current);
            assert_eq!(owner.resources.consumed_capacity_observation(owner.runtime_instance_id), (0, 0));
            let baseline = owner.resources.retained_capacity_bytes();
            let first = ConsumedOutputEvidence::restored_with_witness_for_test(
                owner, Arc::clone(&identity), facts.facts().clone(), Arc::clone(&basis), Arc::clone(&witness),
            );
            let first_bytes = owner.resources.retained_capacity_bytes() - baseline;
            let before_second = owner.resources.retained_capacity_bytes();
            let second = ConsumedOutputEvidence::restored_with_witness_for_test(
                owner, Arc::clone(&identity), facts.facts().clone(), Arc::clone(&basis), witness,
            );
            let second_bytes = owner.resources.retained_capacity_bytes() - before_second;
            assert!(second_bytes > 0, "another reader still funds its own basis and metadata");
            assert!(second_bytes < first_bytes,
                "another reader of the SAME original fact backing must not fund that backing twice: first={first_bytes}, second={second_bytes}");

            (snapshot_handle, basis, identity, facts, first, second)
        });
        first.establish_restored_before_commit(
                &execution, &handle.source_owner, &mut owner.edit_admission(),
            ).unwrap();
            assert!(matches!(owner.currentness(&basis, &identity, &mut owner.edit_admission()).unwrap(),
                SourceSettlementCurrentness::Clean));
            assert!(first.claim_verified_checkpoint_root(owner, &mut owner.edit_admission()).unwrap(),
                "actual restored prerequisite custody must be established, not bypassed");
        handle.with_runtime_mut(&execution, |runtime| {
            assert_eq!(ConsumedOutputEvidence::verify_many_with_admission(
                &execution, &[first.clone(), second.clone()], owner, runtime, &snapshot_handle,
                &basis, &mut owner.edit_admission(),
            ).unwrap(), ConsumedOutputVerification::Current);
            let (current_handle, current_basis) = snapshot(runtime);
            assert_eq!(current_basis, *basis,
                "custody and currentness do not publish another native head");
            runtime.snapshots().release_snapshot(&current_handle).unwrap();
        });
            drop(first);
            drop(second);
            assert!(owner.resources.consumed_capacity_observation(owner.runtime_instance_id).1 > 0,
                "the restored mark keeps the very same fact ticket after both readers leave");
            assert!(matches!(owner.currentness(&basis, &identity, &mut owner.edit_admission()).unwrap(),
                SourceSettlementCurrentness::Clean));
            assert_eq!(owner.retire_settlements(std::slice::from_ref(&identity), &mut owner.edit_admission()),
                vec![identity]);
            assert_eq!(owner.resources.consumed_capacity_observation(owner.runtime_instance_id), (0, 0),
                "the final mark release refunds original consumed backing even while lineage facts remain");
        handle.with_runtime_mut(&execution, |runtime| {
            runtime.snapshots().release_snapshot(&snapshot_handle).unwrap();
        });
    }).expect("the native custody journey owns its installed host call");
}
