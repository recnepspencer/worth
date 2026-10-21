use super::super::FullVerificationStop;
use super::*;
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

#[test]
fn a_retired_row_absent_in_both_images_does_not_change_the_image() {
    let _oracle =
        crate::facade::primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    retirement_image(request_execution, true);
}

#[test]
fn a_row_disappearing_after_capture_changes_the_image() {
    let _oracle =
        crate::facade::primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    retirement_image(request_execution, false);
}

fn retirement_image(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    retired_before_capture: bool,
) {
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request()))
        .expect("installed host call opens");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected.resolve_entity(
                &phase,
                AccountStatus::reference(),
                "open".to_owned(),
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
        })
        .expect("installed host call opens")
        .unwrap()
        .entity_id();
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: OutputBindingIdentity::declared("StatusOutput"),
    };
    let identity = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    handle.with_runtime_mut(request_execution, |runtime| {
        let (snapshot, basis) = snapshot(runtime);
        register(
            owner,
            Arc::clone(&identity),
            Arc::from([]),
            &basis,
            OrdSet::new(),
        );
        let retire = || {
            assert_eq!(
                owner.retire_settlements(std::slice::from_ref(&identity)),
                vec![Arc::clone(&identity)]
            )
        };

        if retired_before_capture {
            retire();
        }
        let captured = owner
            .capture_full_verification(runtime, &snapshot, &basis)
            .unwrap();

        if !retired_before_capture {
            retire();
        }
        let observed = OrdSet::unit(Arc::clone(&identity));
        assert_eq!(
            captured.fence_semantic_image(&observed),
            if retired_before_capture {
                Ok(())
            } else {
                Err(FullVerificationStop::ActorImageChanged)
            }
        );
        register(owner, identity, Arc::from([]), &basis, OrdSet::new());
        assert_eq!(
            captured.fence_semantic_image(&observed),
            Err(FullVerificationStop::ActorImageChanged)
        );
        drop(captured);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
}
