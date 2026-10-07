//! Native actor consequence with real source snapshots and delivered touches.
//! The test-only relation mint exercises the actor, not producer authorization.

use super::*;
#[cfg(not(feature = "certification-invalidation-equivalence"))]
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerification;
use crate::domain_computation::primary_graph::{
    invariant_projection::{ConsumedOutputEvidence, ConsumedOutputVerificationStop},
    output_lineage::{
        input_cutoff::StableEqualityConsequence,
        invalidation::{
            mark_state::SettlementCurrentness, source_alignment::SnapshotAlignedMarkState,
        },
    },
};

struct OtherUpstream;

#[test]
fn certified_alias_chain_discharge_is_exact_and_later_native_change_repends() {
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
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let source = |output_binding| {
        SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity().clone(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding,
    }
    };
    let a_source = source(TypeId::of::<StatusOutput>());
    let a0 = RecordedSettlementIdentity::retain(&a_source, coordinate, 0);
    let a1 = RecordedSettlementIdentity::retain(&a_source, coordinate, 1);
    let a2 = RecordedSettlementIdentity::retain(&a_source, coordinate, 2);
    let b = RecordedSettlementIdentity::retain(
        &source(TypeId::of::<DownstreamOutput>()),
        coordinate,
        0,
    );
    let c = RecordedSettlementIdentity::retain(&source(TypeId::of::<LateOutput>()), coordinate, 0);
    let other =
        RecordedSettlementIdentity::retain(&source(TypeId::of::<OtherUpstream>()), coordinate, 0);

    handle.with_open_runtime_mut(|runtime| {
        write_field(runtime, entity, label.clone(), "prime");
        let (before_handle, before) = snapshot(runtime);
        let old_a_fact = field_fact(runtime, &before_handle, entity, status.clone());
        register(owner, Arc::clone(&a0), Arc::from([old_a_fact.clone()]), &before, OrdSet::new());
        register(owner, Arc::clone(&other), Arc::from([field_fact(runtime, &before_handle, entity, label.clone())]), &before, OrdSet::new());
        register(owner, Arc::clone(&b), Arc::from([]), &before, OrdSet::unit(Arc::clone(&a0)));
        register(owner, Arc::clone(&c), Arc::from([]), &before, [Arc::clone(&b), Arc::clone(&other)].into_iter().collect());

        write_field(runtime, entity, status.clone(), "closed");
        write_field(runtime, entity, label, "changed");
        let (changed_handle, changed) = snapshot(runtime);
        assert_pending(owner, &changed, &b, &a0);
        assert_pending(owner, &changed, &c, &b);
        assert_pending(owner, &changed, &c, &other);
        let historical_image = owner
            .cell_for_read(&changed, &mut owner.edit_admission())
            .unwrap()
            .unwrap()
            .read_image();

        register_alias(owner, runtime, &changed_handle, &changed, entity, status.clone(), &a0, &a1);
        assert!(matches!(currentness(owner, &changed, &b), SourceSettlementCurrentness::Clean));
        assert_pending(owner, &changed, &c, &b);
        assert_pending(owner, &changed, &c, &other);
        let prior = SnapshotAlignedMarkState::observe_image(&historical_image, &changed).unwrap();
        assert!(matches!(prior.currentness(&b), SettlementCurrentness::PendingUpstream(edges) if edges.contains(&a0)));
        assert_source_only_alias_verification(owner, runtime, &changed_handle, &changed, &before, &a0, &old_a_fact);

        write_field(runtime, entity, status.clone(), "again");
        let (next_handle, next) = snapshot(runtime);
        assert_pending(owner, &next, &b, &a0);
        register_alias(owner, runtime, &next_handle, &next, entity, status.clone(), &a1, &a2);
        assert!(matches!(currentness(owner, &next, &b), SourceSettlementCurrentness::Clean));
        assert_source_only_alias_verification(owner, runtime, &next_handle, &next, &before, &a0, &old_a_fact);

        write_field(runtime, entity, status, "changed-output");
        let (later_handle, later) = snapshot(runtime);
        assert_pending(owner, &later, &b, &a0);
        assert!(matches!(
            currentness(owner, &later, &a2),
            SourceSettlementCurrentness::Dirty(_)
        ));
        assert_eq!(
            ConsumedOutputEvidence::verify_candidate_with_admission(
                &a0,
                std::slice::from_ref(&old_a_fact),
                &[],
                None,
                &before,
                owner,
                runtime,
                &later_handle,
                &later,
                &mut owner.edit_admission(),
            ),
            Err(ConsumedOutputVerificationStop::PendingUpstream),
        );
        for handle in [before_handle, changed_handle, next_handle, later_handle] {
            runtime.snapshots().release_snapshot(&handle).unwrap();
        }
    });
}

fn register_alias(
    owner: &SourceInvalidationOwner,
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
    selected: &PositionedRelationalSnapshot,
    entity: EntityId,
    locator: AspectFieldLocator,
    predecessor: &Arc<RecordedSettlementIdentity>,
    successor: &Arc<RecordedSettlementIdentity>,
) {
    let relation = StableEqualityConsequence::native_actor_fixture(
        Arc::clone(predecessor),
        Arc::clone(successor),
        selected,
    );
    let prepared = owner
        .prepare_current_stable_settlement(
            SettlementRegistration {
                work_membership: None,
                identity: Arc::clone(successor),
                facts: Arc::from([field_fact(runtime, handle, entity, locator)]),
                output_facts: None,
                read_basis: selected.clone(),
                stale_at_read_basis: OrdSet::new(),
                requirement: None,
                upstream: OrdSet::new(),
            },
            selected,
            relation,
            &mut owner.edit_admission(),
        )
        .expect("the exact selected alias prepares");
    let cleanup = match prepared.install() {
        Ok(cleanup) => cleanup,
        Err(_) => panic!("the exact selected source image must install"),
    };
    drop(cleanup);
}

fn assert_pending(
    owner: &SourceInvalidationOwner,
    selected: &PositionedRelationalSnapshot,
    target: &Arc<RecordedSettlementIdentity>,
    predecessor: &Arc<RecordedSettlementIdentity>,
) {
    match currentness(owner, selected, target) {
        SourceSettlementCurrentness::PendingUpstream(edges) => assert!(edges.contains(predecessor)),
        _ => panic!("only the exact consumed predecessor can be pending"),
    }
}

fn assert_source_only_alias_verification(
    owner: &SourceInvalidationOwner,
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
    selected: &PositionedRelationalSnapshot,
    observed: &PositionedRelationalSnapshot,
    old_identity: &Arc<RecordedSettlementIdentity>,
    old_fact: &WorthQueryApplicationObservedFact,
) {
    let verified = ConsumedOutputEvidence::verify_candidate_with_admission(
        old_identity,
        std::slice::from_ref(old_fact),
        &[],
        None,
        observed,
        owner,
        runtime,
        handle,
        selected,
        &mut owner.edit_admission(),
    );
    #[cfg(not(feature = "certification-invalidation-equivalence"))]
    assert_eq!(verified, Ok(ConsumedOutputVerification::Current));
    #[cfg(feature = "certification-invalidation-equivalence")]
    assert_eq!(
        verified,
        Err(ConsumedOutputVerificationStop::Unavailable),
        "actor-only aliases cannot supply the missing performed output projection",
    );
}
