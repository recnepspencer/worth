use super::super::FullVerificationStop;
use super::*;
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

#[test]
fn a_retired_row_absent_in_both_images_does_not_change_the_image() {
    retirement_image(true);
}

#[test]
fn a_row_disappearing_after_capture_changes_the_image() {
    retirement_image(false);
}

fn retirement_image(retired_before_capture: bool) {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
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
    handle.with_runtime_mut(|runtime| {
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
                owner.retire_settlements(
                    std::slice::from_ref(&identity),
                    &mut owner.edit_admission()
                ),
                vec![Arc::clone(&identity)]
            )
        };
        if retired_before_capture {
            retire();
        }
        let captured = owner
            .capture_full_verification(runtime, &snapshot, &basis, &mut owner.edit_admission())
            .unwrap();
        if !retired_before_capture {
            retire();
        }
        let observed = OrdSet::unit(Arc::clone(&identity));
        assert_eq!(
            captured.fence_semantic_image(&observed, &mut owner.edit_admission()),
            if retired_before_capture {
                Ok(())
            } else {
                Err(FullVerificationStop::ActorImageChanged)
            }
        );
        register(owner, identity, Arc::from([]), &basis, OrdSet::new());
        assert_eq!(
            captured.fence_semantic_image(&observed, &mut owner.edit_admission()),
            Err(FullVerificationStop::ActorImageChanged)
        );
        drop(captured);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
}

#[test]
fn shared_empty_images_still_fence_a_changed_native_position() {
    use worth_foundational::facade::{AspectValue, InternedString};
    use worth_relational::facade::{
        mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent},
        transactions::{
            AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
            WorkerIntentBatch,
        },
    };
    let world = installed_authorization_world(true);
    let entity = world
        .selected_product()
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let field = AccountLabel::reference();
    let locator = graph
        .layout()
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let resources = &owner.resources;
    let held = resources
        .reserve_retained_capacity(
            resources.installation().maximum_retained_bytes - resources.retained_capacity_bytes(),
        )
        .unwrap();
    handle.with_runtime_mut(|runtime| {
        write_field(runtime, entity, locator.clone(), "first-discard");
        let (snapshot, basis) = snapshot(runtime);
        let first = owner
            .cell_for_read(&basis, &mut owner.edit_admission())
            .unwrap()
            .unwrap()
            .read_image();
        assert!(first.payload().current.settlements.is_empty());
        let native_basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        let mut transaction = runtime
            .begin_branch_transaction(&native_basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("second-discard").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                            locator,
                            AspectValue::String(InternedString::Raw("second-discard".to_owned())),
                        )])),
                    }),
                )),
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        // The first discard released its predecessor's retained history.
        // Fill that new headroom so the second publication also discards.
        let second_held = resources
            .reserve_retained_capacity(
                resources.installation().maximum_retained_bytes
                    - resources.retained_capacity_bytes(),
            )
            .unwrap();
        let candidate = runtime
            .prepare_branch_transaction(
                transaction,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let captured = owner
            .capture_full_verification(runtime, &snapshot, &basis, &mut owner.edit_admission())
            .unwrap();
        let outcome = runtime.publication_port().compare_and_publish(candidate);
        let RelationalPublicationOutcome::Performed(performed) = outcome else {
            panic!("the second discard must publish: {outcome:?}");
        };
        let second = owner
            .cell_for_read(&basis, &mut owner.edit_admission())
            .unwrap()
            .unwrap()
            .read_image();
        assert!(
            Arc::ptr_eq(first.payload(), second.payload()),
            "both discards share the funded vacant image"
        );
        assert_ne!(first.root_id(), second.root_id());
        assert_eq!(
            captured.fence_semantic_image(&OrdSet::new(), &mut owner.edit_admission()),
            Err(FullVerificationStop::Alignment(
                super::super::FullVerificationReason::RetainedDeliveryGap
            )),
            "exact Native position selection precedes the pointer fence"
        );
        drop(captured);
        let committed = runtime.settle_performed_publication(performed).unwrap();
        release_test_commit_snapshot(runtime, &committed);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
        drop(second_held);
    });
    drop(held);
}
