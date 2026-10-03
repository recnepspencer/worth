//! Stable preparation cannot reinterpret a cutoff at an older source root.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop;
use worth_relational::facade::mvcc::CompanionCellEditStop;

#[test]
fn current_source_registration_rejects_native_movement_while_ordinary_replay_remains_valid() {
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
    let observation = product.observation();
    let coordinate = ProductCoordinate {
        occurrence: observation.lifecycle_incarnation(),
        generation: observation.reference_generation().get(),
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
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    handle.with_runtime_mut(|runtime| write_field(runtime, entity, label, "prime"));
    let (before_handle, before, facts) = handle.with_runtime(|runtime| {
        let (snapshot, basis) = snapshot(runtime);
        let facts: Arc<[_]> = Arc::from([field_fact(runtime, &snapshot, entity, status.clone())]);
        (snapshot, basis, facts)
    });
    let identity = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let registration = |identity| SettlementRegistration {
        work_membership: None,
        identity,
        facts: Arc::clone(&facts),
        output_facts: None,
        read_basis: before.clone(),
        requirement: None,
        upstream: OrdSet::new(),
    };
    let prepared = owner
        .prepare_current_settlement(
            registration(Arc::clone(&identity)),
            &before,
            &mut owner.edit_admission(),
        )
        .expect("the exact current native source can prepare");
    assert!(Arc::ptr_eq(prepared.identity(), &identity));
    assert!(std::ptr::eq(prepared.selected_source(), &before));
    let cleanup = prepared
        .install()
        .unwrap_or_else(|_| panic!("an unchanged source image installs its prepared registration"));
    drop(cleanup);
    assert!(matches!(
        currentness(owner, &before, &identity),
        SourceSettlementCurrentness::Clean
    ));

    handle.with_runtime_mut(|runtime| write_field(runtime, entity, status, "closed"));
    let (after_handle, after) = handle.with_runtime(snapshot);
    let next = RecordedSettlementIdentity::retain(&source, coordinate, 1);
    let stopped = owner.prepare_current_settlement(
        registration(Arc::clone(&next)),
        &before,
        &mut owner.edit_admission(),
    );
    assert!(matches!(
        stopped,
        Err(SettlementRegistrationStop::Edit(
            CompanionCellEditStop::TopologyGenerationChanged
        ))
    ));
    assert!(matches!(
        currentness(owner, &after, &next),
        SourceSettlementCurrentness::FullVerificationRequired(
            super::super::FullVerificationReason::MissingSettlement
        )
    ));
    // An ordinary post-effect registration must still replay deliveries from
    // its retained basis rather than pretending its old fact is current.
    owner
        .register_settlement(registration(Arc::clone(&next)), &mut owner.edit_admission())
        .unwrap();
    match currentness(owner, &after, &next) {
        SourceSettlementCurrentness::Dirty(ordinals) => assert!(ordinals.contains(&0)),
        _ => panic!("ordinary retained registration must mark the changed source fact"),
    }
    handle.with_runtime_mut(|runtime| {
        runtime
            .snapshots()
            .release_snapshot(&before_handle)
            .unwrap();
        runtime.snapshots().release_snapshot(&after_handle).unwrap();
    });
}
