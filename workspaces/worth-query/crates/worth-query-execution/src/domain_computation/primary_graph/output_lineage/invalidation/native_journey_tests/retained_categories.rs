//! Actual indexed registration, prior images, capacity refusal and retry.
use super::super::derived::SettlementRegistrationStop;
use super::*;
use worth_relational::facade::mvcc::CompanionPreflightStop;

#[test]
fn actual_registration_keeps_facts_and_prior_images_but_refuses_over_capacity_before_install() {
    let world = super::marking_ceiling::world_installing(|defaults| {
        crate::domain_computation::execution_runtime::WorthQueryInvalidationResourceInstallation {
            maximum_preparation_bytes: 256 * 1024 * 1024,
            maximum_marking_work: 1_000_000,
            maximum_retained_bytes: 256 * 1024,
            ..defaults
        }
    });
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".into(),
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
        output_binding: TypeId::of::<StatusOutput>(),
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
    let (snapshot_handle, basis, fact) = handle.with_runtime(|runtime| {
        let (snapshot_handle, basis) = snapshot(runtime);
        let fact = field_fact(runtime, &snapshot_handle, entity, status);
        (snapshot_handle, basis, fact)
    });
    let first = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let second = RecordedSettlementIdentity::retain(&source, coordinate, 1);
    let registration = |identity, count| SettlementRegistration {
        work_membership: None,
        identity,
        facts: Arc::from(vec![fact.clone(); count]),
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
    assert!(admission.charged_bytes() > admission.charged_index_bytes());
    let current = cell.read_image();
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
    handle.with_runtime_mut(|runtime| {
        runtime
            .snapshots()
            .release_snapshot(&snapshot_handle)
            .unwrap()
    });
}
