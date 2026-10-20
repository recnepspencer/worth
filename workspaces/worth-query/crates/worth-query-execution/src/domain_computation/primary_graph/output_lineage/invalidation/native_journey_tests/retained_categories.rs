//! Actual indexed registration, prior images, capacity refusal and retry.
use super::super::derived::SettlementRegistrationStop;
use super::*;
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use worth_relational::facade::mvcc::CompanionPreflightStop;

#[test]
fn late_capacity_fallback_does_not_accept_the_provisional_exact_marking() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world = super::marking_ceiling::world_installing(|defaults| {
        crate::domain_computation::execution_runtime::WorthQueryInvalidationResourceInstallation {
            maximum_retained_bytes: Some(128 * 1024 * 1024),
            ..defaults
        }
    });
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request();
        let selected = world.selected_product(&custody);
        let entity = selected.resolve_entity(&phase, AccountStatus::reference(),
            "open".to_owned(), &live_scope(), WorthQueryPrincipalResolutionMode::Ordinary)
            .unwrap().entity_id();
        let (_, product, _) = selected.into_parts();
        let source = SemanticSource {
            runtime_authority: world.application.runtime.authority_identity().as_u64(),
            schema: world.application.installed_schema.binding_identity(),
            scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
            output_binding: OutputBindingIdentity::declared("StatusOutput"),
        };
        let output = RecordedSettlementIdentity::retain(&source, ProductCoordinate {
            occurrence: product.observation().lifecycle_incarnation(),
            generation: product.observation().reference_generation().get(),
        }, 0);
        let graph = world.application.runtime.primary_graph().unwrap();
        let field = AccountStatus::reference();
        let locator = graph.layout().field_locator(field.entity(), field.aspect(), field.field())
            .unwrap().clone();
        let handle = graph.integration_handle();
        let owner = &handle.source_owner.invalidation_owner;
        handle.with_runtime_mut(execution, |runtime| {
            let (before_handle, before) = snapshot(runtime);
            let old = field_fact(&custody.borrowed_request_for_test(), runtime,
                &before_handle, entity, locator.clone());
            register(owner, output.clone(), Arc::from([old.clone()]), &before, OrdSet::new());
            let cell = owner.cell_for_read(&before, &mut owner.edit_admission()).unwrap().unwrap();
            let image = cell.read_image();
            // Competing custody leaves one whole-index quote. The partial
            // mark can fit, but after spending that capacity the late guard
            // cannot retain another whole index of headroom for future edits.
            let room = super::super::retention::state_bound(&image.payload().current)
                .unwrap();
            let retained_before = owner.resources.retained_capacity_bytes();
            let remaining = owner.resources.installation().maximum_retained_bytes.expect("this court installs a finite retained ceiling") - retained_before;
            let competing = owner.resources.reserve_retained_capacity(remaining.checked_sub(room).unwrap()).unwrap();
            let _observation = crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
            super::super::marking_observation::native_marking_on_this_thread_for_test();
            write_field(execution, runtime, entity, locator.clone(), "closed");
            let (after_handle, after) = snapshot(runtime);
            let commit = after.commit_id().unwrap();
            let reports = super::super::marking_observation::native_marking_on_this_thread_for_test();
            let attempts: Vec<_> = reports.iter().filter(|report| report.commit == commit).collect();
            let provisional = attempts.iter().position(|report| matches!(report.precision,
                super::super::logical_marking::WorthQueryNativeMarkingPrecision::Exact(_)))
                .expect("the actual marking completed before later capacity admission failed");
            let fallback = attempts.iter().position(|report| matches!(report.precision,
                super::super::logical_marking::WorthQueryNativeMarkingPrecision::RetainedCapacityExhausted(_))
                && report.charged_work.is_none())
                .expect("the same native commit prepared the empty-image fallback");
            assert!(provisional < fallback, "provisional work precedes fallback selection: {reports:?}");
            assert!(attempts[provisional].charged_work.is_some_and(|work| work > 0));
            assert!(attempts[provisional].affected.as_ref().is_some_and(|outputs|
                outputs.iter().any(|marked| marked.root == source.scope)));
            assert!(attempts[fallback].affected.is_none(), "unknown affected outputs are not an exact empty set");
            // Acceptance comes from the installed native image and source
            // snapshot, independently of either prepared diagnostic report.
            assert!(owner.native_marking_report(&after, &mut owner.edit_admission()).unwrap().is_none(),
                "the accepted empty image does not accept the provisional exact report");
            assert!(matches!(currentness(owner, &after, &output),
                SourceSettlementCurrentness::FullVerificationRequired(super::super::mark_state::FullVerificationReason::MissingSettlement)),
                "the accepted fallback requires genuine re-verification");
            assert_ne!(field_fact(&custody.borrowed_request_for_test(), runtime,
                &after_handle, entity, locator.clone()), old, "the source write really published");
            assert_eq!(field_fact(&custody.borrowed_request_for_test(), runtime,
                &before_handle, entity, locator), old, "the original source snapshot remains intact");
            drop(competing);
            assert_eq!(owner.resources.retained_capacity_bytes(), retained_before,
                "discarded preparation and competing custody return their real reservations while the original image stays held");
            for snapshot in [before_handle, after_handle] {
                runtime.snapshots().release_snapshot(&snapshot).unwrap();
            }
        });
    }).expect("the native court owns its installed host call");
}

#[test]
fn actual_registration_keeps_facts_and_prior_images_but_refuses_over_capacity_before_install() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = super::marking_ceiling::world_installing(|defaults| {
        crate::domain_computation::execution_runtime::WorthQueryInvalidationResourceInstallation {
            maximum_preparation_bytes: None,
            maximum_marking_work: Some(1_000_000),
            maximum_retained_bytes: Some(256 * 1024),
            ..defaults
        }
    });
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
                    "open".into(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id()
        })
        .expect("the fixture read owns its installed host call");
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
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let (snapshot_handle, basis, fact) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            handle.with_runtime(request_execution, |runtime| {
                let (snapshot_handle, basis) = snapshot(runtime);
                let fact = field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &snapshot_handle,
                    entity,
                    status,
                );
                (snapshot_handle, basis, fact)
            })
        })
        .expect("the fixture read owns its installed host call");
    let first = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let second = RecordedSettlementIdentity::retain(&source, coordinate, 1);
    let registration = |identity, count| SettlementRegistration {
        work_membership: None,
        identity,
        facts:
            crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(
                false,
                Arc::from(vec![fact.clone(); count]),
            ),
        output_facts: None,
        read_basis: basis.clone(),
        stale_at_read_basis: OrdSet::new(),
        requirement: None,
        upstream: OrdSet::new(),
    };
    owner
        .register_settlement(
            registration(Arc::clone(&first), 1),
            &mut owner.edit_admission(),
        )
        .unwrap();
    let cell = owner
        .cell_for_read(&basis, &mut owner.edit_admission())
        .unwrap()
        .unwrap();
    let prior = cell.read_image();
    let retained = owner.resources.retained_capacity_bytes();
    assert!(matches!(
        owner.prepare_current_settlement(
            registration(Arc::clone(&second), 512),
            &basis,
            &mut owner.edit_admission()
        ),
        Err(SettlementRegistrationStop::Admission(
            CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }
        ))
    ));
    assert_eq!(owner.resources.retained_capacity_bytes(), retained);
    assert!(Arc::ptr_eq(cell.read_image().payload(), prior.payload()));
    assert!(!prior.payload().current.settlements.contains_key(&second));
    let mut admission = owner.edit_admission();
    owner
        .register_settlement(registration(Arc::clone(&second), 1), &mut admission)
        .unwrap();
    // Preparation and retained index storage are separate reservations, so
    // neither total orders the other. Preparation covers the new state and
    // root objects, which are never index storage. The replacing image
    // reserves the replaced version's reservation plus what this edit copied
    // into the retained index, never more than its whole index.
    let capacity = super::super::index_capacity::arc_bytes::<
        crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity,
    >()
    .unwrap();
    let reserved = |state: &super::super::mark_state::MarkState| {
        state.retained_capacity.as_ref().unwrap().bytes() - capacity
    };
    assert!(
        admission.charged_bytes()
            >= super::super::index_capacity::arc_bytes::<super::super::mark_state::MarkState>()
                .unwrap()
                + super::super::index_capacity::arc_bytes::<
                    super::super::source_alignment::BranchMarkRoot,
                >()
                .unwrap()
    );
    let copied = admission.charged_index_bytes();
    assert!(copied > 0);
    let current = cell.read_image();
    let whole = super::super::retention::state_bound(&current.payload().current).unwrap();
    let (own, replaced) = (
        reserved(&current.payload().current),
        reserved(&prior.payload().current),
    );
    assert_eq!(own, (replaced + copied).min(whole));
    assert!(own > replaced);
    assert_eq!(current.payload().current.settlements.len(), 2);
    assert_eq!(prior.payload().current.settlements.len(), 1);
    assert_eq!(
        current
            .payload()
            .current
            .settlements
            .get(&second)
            .unwrap()
            .facts
            .len(),
        1
    );
    let before = admission.charged_index_bytes();
    assert_eq!(
        owner.retire_settlements(&[Arc::clone(&first)], &mut admission),
        vec![Arc::clone(&first)]
    );
    assert!(admission.charged_index_bytes() > before);
    let after = cell.read_image();
    assert_eq!(after.payload().current.settlements.len(), 1);
    assert!(after.payload().current.settlements.contains_key(&second));
    assert!(after
        .payload()
        .current
        .postings
        .values()
        .all(|postings| postings.len() == 1));
    assert_eq!(prior.payload().current.settlements.len(), 1);
    // The whole-index cap includes any later basis carries and refuses overflow.
    let mut overflowing = (*after.payload().current).clone();
    overflowing.maximum_basis_allocation_bytes = u64::MAX;
    assert!(super::super::retention::state_bound(&overflowing).is_none());
    assert!(Arc::ptr_eq(cell.read_image().payload(), after.payload()));
    // Removing the last user drops its posting bucket, while the retained
    // nonempty image remains usable and owns its original tree and fact.
    assert_eq!(
        owner.retire_settlements(&[Arc::clone(&second)], &mut admission),
        vec![Arc::clone(&second)]
    );
    let empty = cell.read_image();
    assert!(empty.payload().current.postings.is_empty());
    assert!(empty.payload().current.settlements.is_empty());
    assert_eq!(after.payload().current.settlements.len(), 1);
    assert!(after
        .payload()
        .current
        .postings
        .values()
        .all(|postings| postings.len() == 1));
    handle.with_runtime_mut(request_execution, |runtime| {
        runtime
            .snapshots()
            .release_snapshot(&snapshot_handle)
            .unwrap()
    });
}
