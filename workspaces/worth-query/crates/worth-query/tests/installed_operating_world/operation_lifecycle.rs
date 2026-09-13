use worth_foundational::facade::{FieldKey, InternedString};
use worth_query::facade::{domain, foundation, read};

use super::installed_operation_fixture::{
    configured_runtime, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};

type SettledProjection = domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

#[test]
fn settled_projection_promotes_refreshes_and_closes_on_drop() {
    let mut workspace = configured_runtime()
        .workspace("projection-lifecycle-success")
        .unwrap();
    let (settled, native_key) = settle_native(&mut workspace);
    let settled_identity = settled.identity().to_string();
    let current = settled.into_lifecycle();
    let predecessor_identity = current.identity().to_string();
    let live = match current.promote(&mut workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("current settled projection did not promote"),
    };

    assert_eq!(live.receipt().settled_identity(), settled_identity);
    assert_eq!(live.predecessor_identity(), predecessor_identity);
    assert_eq!(live.receipt().counters().lifecycle_attempts, 1);
    assert_eq!(live.receipt().counters().fresh_conditional_decisions, 0);
    assert_eq!(live.receipt().counters().planning_attempts, 1);
    assert_eq!(live.receipt().counters().planning_completions, 1);
    assert_eq!(live.receipt().counters().lower_runtime_contacts, 1);
    assert_eq!(live.receipt().counters().managed_resource_registrations, 1);

    let refresh = live.refresh(&mut workspace).unwrap();
    assert_eq!(refresh.work().authority_checks(), 1);
    assert_eq!(refresh.work().drain_calls(), 1);
    assert_eq!(refresh.work().delivery_batches(), 0);
    assert_eq!(refresh.work().maintenance_batches(), 0);
    assert_eq!(refresh.work().read_calls(), 1);
    assert_eq!(refresh.work().projection_calls(), 1);
    assert_eq!(refresh.work().native_rebind_calls(), 1);
    assert_eq!(
        refresh.authority().consumer_contract(),
        live.snapshot().authority().consumer_contract()
    );
    let native = refresh.native_value(&native_key, 0).unwrap();
    assert_eq!(
        native.fact().as_interned_string(),
        Ok(&InternedString::Raw("synthetic-anchor".into()))
    );
    assert_eq!(native.counters().indexed_accesses, 1);
    assert_eq!(native.counters().fact_scans, 0);
    let inserted = workspace
        .insert("Vertex", |mutation| {
            mutation.aspect("identity.id", "live-update")
        })
        .unwrap();
    let refreshed = live.refresh(&mut workspace).unwrap();
    assert_eq!(
        refreshed.impact().class(),
        domain::WorthQueryImpactClass::ValuePatch
    );
    assert_eq!(refreshed.impact().counters().unrelated_dependency_scans, 0);
    assert_eq!(refreshed.work().delivery_batches(), 1);
    assert_eq!(refreshed.work().maintenance_batches(), 1);
    assert_eq!(refreshed.work().mutation_deltas(), 1);
    assert_eq!(refreshed.work().live_view_updates(), 1);
    assert_eq!(refreshed.delivery().batches().len(), 1);
    let maintenance = refreshed.delivery().batches()[0]
        .maintenance_work()
        .expect("committed insert must retain live maintenance evidence");
    assert_eq!(maintenance.mutation_delta_count(), 1);
    assert_eq!(maintenance.live_view_update_count(), 1);
    let updated = refreshed.native_value(&native_key, 0).unwrap();
    assert_eq!(
        updated.fact().as_interned_string(),
        Ok(&InternedString::Raw("live-update".into()))
    );
    assert_eq!(updated.counters().row_scans, 0);
    let inserted_identity = inserted.deltas()[0].entity_identity().clone();
    workspace
        .update(inserted_identity.clone(), |mutation| {
            mutation.aspect("identity.id", "live-updated-again")
        })
        .unwrap();
    let updated_refresh = live.refresh(&mut workspace).unwrap();
    assert_eq!(
        updated_refresh.impact().class(),
        domain::WorthQueryImpactClass::ValuePatch
    );
    assert_eq!(updated_refresh.delivery().batches().len(), 1);
    workspace.delete(inserted_identity).unwrap();
    let deleted_refresh = live.refresh(&mut workspace).unwrap();
    assert_eq!(
        deleted_refresh.impact().class(),
        domain::WorthQueryImpactClass::Retirement
    );
    assert_eq!(deleted_refresh.delivery().batches().len(), 1);
    let resource_name = live.resource_name().to_string();
    workspace
        .resolve_live_artifact_target(&resource_name)
        .unwrap();
    drop(live);
    assert!(workspace
        .resolve_live_artifact_target(&resource_name)
        .is_err());
}

#[test]
fn foreign_runtime_denial_preserves_the_projection_for_owner_retry() {
    let mut owner = configured_runtime()
        .workspace("projection-lifecycle-owner")
        .unwrap();
    let settled = settle(&mut owner);
    let mut foreign = configured_runtime()
        .workspace("projection-lifecycle-foreign")
        .unwrap();
    let stop = match settled.into_lifecycle().promote(&mut foreign) {
        domain::WorthQueryProjectionPromotionOutcome::Denied(stop) => stop,
        _ => panic!("foreign runtime did not deny promotion"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryProjectionPromotionDenialKind::ForeignRuntime
    );
    assert_no_attempt_or_planning(stop.counters());

    assert!(matches!(
        stop.into_current().promote(&mut owner),
        domain::WorthQueryProjectionPromotionOutcome::Promoted(_)
    ));
}

#[test]
fn stale_installation_and_unsupported_basis_stop_before_lifecycle_work() {
    let mut controlled = configured_runtime()
        .controlled_workspace("projection-lifecycle-stale")
        .unwrap();
    let settled = settle(&mut controlled);
    controlled.advance_domain_installation_generation().unwrap();
    let stale = match settled.into_lifecycle().promote(&mut controlled) {
        domain::WorthQueryProjectionPromotionOutcome::Stale(stale) => stale,
        _ => panic!("replaced installation generation did not produce stale proof"),
    };
    assert_no_attempt_or_planning(stale.counters());
    assert!(!stale.snapshot().identity().is_empty());

    let workspace = configured_runtime()
        .workspace("projection-lifecycle-wrong-basis")
        .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let branch = workspace
        .branches()
        .fork(workspace.current_world())
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create()
        .unwrap();
    let denial = match workspace
        .observe_operating_world(branch)
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
    {
        Err(denial) => denial,
        Ok(_) => panic!("unsupported branch basis minted a lifecycle-capable binding"),
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::BasisExecutionUnsupported
    );
    assert_eq!(denial.counters().planning_steps, 0);
}

#[test]
fn live_refresh_revalidates_installation_before_maintenance_or_delivery() {
    let mut workspace = configured_runtime()
        .controlled_workspace("projection-lifecycle-live-drift")
        .unwrap();
    let settled = settle(&mut workspace);
    let live = match settled.into_lifecycle().promote(&mut workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("current projection did not promote before drift"),
    };
    workspace.advance_domain_installation_generation().unwrap();

    let stop = match live.refresh(&mut workspace) {
        Err(domain::WorthQueryLiveProjectionRefreshError::Authority(stop)) => stop,
        _ => panic!("stale live projection reached maintenance or delivery"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryDomainHandleDenialKind::StaleInstallationGeneration
    );
    assert_eq!(stop.work().drain_calls(), 0);
    assert_eq!(stop.work().delivery_batches(), 0);
    assert_eq!(stop.work().read_calls(), 0);
    assert_eq!(stop.work().projection_calls(), 0);
    assert_eq!(stop.work().native_rebind_calls(), 0);
}

#[test]
fn unsupported_live_support_denies_before_conditional_or_planning_work() {
    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Live,
            domain::WorthQueryConsumerSupportPosture::Unsupported,
        )
        .workspace("projection-lifecycle-unsupported-live")
        .unwrap();
    let settled = settle(&mut workspace);
    let stop = match settled.into_lifecycle().promote(&mut workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Denied(stop) => stop,
        _ => panic!("unsupported live support did not deny promotion"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryProjectionPromotionDenialKind::LiveSupportUnavailable
    );
    assert_eq!(stop.counters().conditional_lowerings_checked, 0);
    assert_no_attempt_or_planning(stop.counters());
}

fn settle(workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace) -> SettledProjection {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(consumer, read::project_facts().entity_identities())
        .unwrap()
        .settle()
        .unwrap()
}

fn settle_native(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> (SettledProjection, domain::WorthQueryNativeAccessKey) {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let mut builder = bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let selection = builder
        .select_display_native_field(FieldKey::new("id").unwrap())
        .unwrap();
    let request = builder.build().unwrap();
    let key = request.resolve_native_key(&selection).unwrap().into_key();
    let settled = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();
    (settled, key)
}

fn assert_no_attempt_or_planning(counters: domain::WorthQueryProjectionPromotionCounters) {
    assert_eq!(counters.lifecycle_attempts, 0);
    assert_eq!(counters.fresh_conditional_decisions, 0);
    assert_eq!(counters.planning_attempts, 0);
    assert_eq!(counters.lower_runtime_contacts, 0);
    assert_eq!(counters.managed_resource_registrations, 0);
}
