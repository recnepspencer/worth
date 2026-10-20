//! Native source custody, shared settlement installation, retirement and races.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StorageControl,
    output_lineage::RetainedSourceFacts,
};

#[test]
fn native_projection_continuation_keeps_exact_bindings_and_rejects_source_races() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts();
    let observation = product.observation();
    let coordinate = ProductCoordinate {
        occurrence: observation.lifecycle_incarnation(),
        generation: observation.reference_generation().get(),
    };
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: OutputBindingIdentity::declared("StatusOutput"),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    handle.with_runtime_mut(request_execution, |runtime| {
        write_field(request_execution, runtime, entity, label, "prime")
    });
    let (before_handle, before, facts) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            handle.with_runtime(request_execution, |runtime| {
                let (snapshot, basis) = snapshot(runtime);
                let facts: Arc<[_]> = Arc::from([field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &snapshot,
                    entity,
                    status.clone(),
                )]);
                (snapshot, basis, facts)
            })
        })
        .expect("the fixture read owns its installed host call");

    let first = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    owner
        .register_settlement(
            SettlementRegistration {
                computation: Some(super::super::ComputationRegistration::retained()),
                work_membership: None,
                identity: first.clone(),
                facts: RetainedSourceFacts::for_test(false, facts.clone()),
                output_facts: None,
                read_basis: before.clone(),
                stale_at_read_basis: OrdSet::new(),
                requirement: None,
                upstream: OrdSet::new(),
            },
            &mut owner.edit_admission(),
        )
        .unwrap();
    let cell = owner
        .cell_for_read(&before, &mut owner.edit_admission())
        .unwrap()
        .unwrap();
    let historical = cell.read_image();
    let baseline_capacity = historical.payload().current.settlements[&first]
        .facts
        .postconditions()
        .lock_retention()
        .allocation
        .clone();
    let original = historical.payload().current.settlements[&first]
        .facts
        .clone();
    let original_body = original.postconditions().clone();
    let continuation = handle.with_runtime_mut(request_execution, |runtime| {
        let changes = owner
            .computation_changes(
                runtime,
                &before_handle,
                &first,
                &original,
                &mut owner.edit_admission(),
            )
            .unwrap()
            .unwrap()
            .bind_original(original_body.clone());
        let control = StorageControl::new(request_execution.allocation_policy(), None);
        let carry = changes
            .carry(
                &original_body,
                original_body.len(),
                &OrdSet::new(),
                &before_handle,
                control,
            )
            .unwrap()
            .unwrap();
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("posting-correspondence").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields: AspectFieldPatch::from(BTreeMap::from([(
                            status.clone(),
                            AspectValue::String(InternedString::Raw("closed".to_owned())),
                        )])),
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let candidate = runtime
            .prepare_branch_transaction(request_execution, transaction)
            .unwrap();
        assert!(carry
            .matches_publication_source(runtime, &before_handle, &mut owner.edit_admission())
            .unwrap());
        let selection = candidate
            .with_prepared_descriptive_touches(|touches| {
                carry.select(touches, original_body.len(), &[], control)
            })
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            selection.ordinals.contains(&0),
            "the actual candidate selects the native field read"
        );
        drop(candidate);
        // This owner proof seals unchanged revisions at the same live image;
        // it does not claim a performed producer or original-read permission.
        selection
            .basis
            .seal(
                original_body.clone(),
                before.clone(),
                selection.memory,
                control,
            )
            .unwrap()
    });
    let held_continuation = continuation.clone();
    let second = RecordedSettlementIdentity::retain(&source, coordinate, 1);
    let registration = |identity, body, continuation| SettlementRegistration {
        computation: None,
        work_membership: None,
        identity,
        facts: RetainedSourceFacts::for_test(false, body).with_posting_continuation(continuation),
        output_facts: None,
        read_basis: before.clone(),
        stale_at_read_basis: OrdSet::new(),
        requirement: None,
        upstream: OrdSet::new(),
    };
    // Exact completed-body correspondence, not data equality, permits sharing.
    owner
        .register_settlement(
            registration(second.clone(), facts.clone(), Some(continuation.clone())),
            &mut owner.edit_admission(),
        )
        .unwrap();
    let installed = cell.read_image();
    let a = Arc::clone(&installed.payload().current.settlements[&first].posting_projection);
    let b = Arc::clone(&installed.payload().current.settlements[&second].posting_projection);
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(installed.payload().current.posting_bindings.len(), 1);
    assert_eq!(
        installed.payload().current.posting_bindings[&a.id]
            .settlements()
            .unwrap()
            .settlements
            .len(),
        2
    );
    let unrelated = Arc::from(facts.to_vec().into_boxed_slice());
    let third = RecordedSettlementIdentity::retain(&source, coordinate, 2);
    owner
        .register_settlement(
            registration(third.clone(), unrelated, Some(continuation.clone())),
            &mut owner.edit_admission(),
        )
        .unwrap();
    let reconstructed = cell.read_image();
    assert!(
        !Arc::ptr_eq(
            &reconstructed.payload().current.settlements[&third].posting_projection,
            &a
        ),
        "equal unrelated backing reconstructs"
    );
    let retired = owner.retire_settlements(&[first.clone()], &mut owner.edit_admission());
    assert!(retired.contains(&first));
    let retired_image = cell.read_image();
    assert!(!retired_image
        .payload()
        .current
        .settlements
        .contains_key(&first));
    assert!(retired_image.payload().current.posting_bindings[&a.id]
        .settlements()
        .unwrap()
        .settlements
        .contains(&second));
    assert!(historical
        .payload()
        .current
        .settlements
        .contains_key(&first));
    assert!(!historical
        .payload()
        .current
        .settlements
        .contains_key(&second));
    // A prepared current image may not overwrite a real intervening native edit.
    let fourth = RecordedSettlementIdentity::retain(&source, coordinate, 3);
    let prepared = owner
        .prepare_settlement(
            registration(fourth.clone(), facts.clone(), Some(continuation)),
            &mut owner.edit_admission(),
        )
        .unwrap();
    handle.with_runtime_mut(request_execution, |runtime| {
        write_field(request_execution, runtime, entity, status, "closed")
    });
    assert!(prepared.install().is_err());
    let (after_handle, after) = handle.with_runtime(request_execution, snapshot);
    assert!(matches!(
        currentness(owner, &after, &second),
        SourceSettlementCurrentness::Dirty(_)
    ));
    assert!(matches!(
        currentness(owner, &after, &fourth),
        SourceSettlementCurrentness::FullVerificationRequired(
            super::super::FullVerificationReason::MissingSettlement
        )
    ));
    drop(retired_image);
    drop(reconstructed);
    drop(installed);
    drop(historical);
    drop(original);
    assert!(baseline_capacity.upgrade().is_some(), "the completed continuation retains its baseline's original direct ticket after old images leave");
    drop(held_continuation);
    assert!(
        baseline_capacity.upgrade().is_none(),
        "no parent mark image or body-history chain retains the old ticket"
    );
    handle.with_runtime_mut(request_execution, |runtime| {
        runtime
            .snapshots()
            .release_snapshot(&before_handle)
            .unwrap();
        runtime.snapshots().release_snapshot(&after_handle).unwrap();
    });
}
