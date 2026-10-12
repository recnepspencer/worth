//! Real issued sessions isolate the native snapshot and cached-root guards.
use crate::domain_computation::primary_graph::application_query::read_execution::read_plan::ReadPlan;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, publish_relational_mutation, AccountLabel,
    AccountStatus, PublicScopedAccountSummaryQuery,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};
use crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

#[test]
fn initial_pin_refuses_another_issued_sessions_snapshot() {
    check(0);
}
#[test]
fn rebound_pin_reads_the_members_own_version_after_a_commit() {
    check(1);
}
#[test]
fn rebound_pin_checks_the_native_version_even_with_matching_cached_basis() {
    check(2);
}
#[test]
fn rebound_pin_checks_cached_basis_even_with_matching_native_version() {
    check(3);
}
#[test]
fn cancelled_pin_returns_the_stop_to_reconstruction() {
    check(4);
}

fn check(case: usize) {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host.borrowed_request_for_test();
            let old = world.selected_product(&host);
            let principal = old
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = old
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let query = world
                .application
                .installed_schema()
                .certification_query(PublicScopedAccountSummaryQuery::reference())
                .unwrap();
            let controls = || {
                WorthQueryProductQueryControls::new(
                    NonZeroUsize::new(10).unwrap(),
                    NonZeroUsize::new(10000).unwrap(),
                    &scope,
                )
            };
            let admit = || {
                old.retain_selection(&execution)
                    .unwrap()
                    .admit_application_query(
                        &phase,
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        controls(),
                    )
                    .unwrap()
            };
            let first = admit();
            let first_read = first
                .graph_work
                .prepare_query_read(
                    first.basis.identity(),
                    &execution,
                    first.basis.snapshot_handle(),
                    |_| vec![],
                )
                .unwrap();
            let mut pin = first_read.retained_pin();
            drop(first_read);
            let same_root = admit();
            let reused = same_root
                .graph_work
                .prepare_query_read_from_pin(
                    same_root.basis.identity(),
                    &execution,
                    same_root.basis.snapshot_handle(),
                    &pin,
                    |_| panic!("equal-root rebind contacted the owner"),
                )
                .unwrap();
            assert!(std::sync::Arc::ptr_eq(&pin, &reused.retained_pin()));
            drop(reused);
            if case == 0 {
                assert_ne!(
                    first.basis.identity().snapshot_id(),
                    same_root.basis.identity().snapshot_id()
                );
                let refusal = first.graph_work.prepare_query_read(
                    first.basis.identity(),
                    &execution,
                    same_root.basis.snapshot_handle(),
                    |_| vec![],
                );
                assert_eq!(
                    refusal.map(|_| ()),
                    Err(WorthQueryManagedGraphReadDenial::ForeignBasis)
                );
                return;
            }
            if case == 4 {
                let cancellation = worth_execution::CancellationSource::new();
                let serial = worth_execution::SerialRequest::from_memory(
                    worth_execution::SerialMemoryBudget::new(16 << 20),
                    cancellation.token(),
                    None,
                );
                cancellation.cancel();
                let request = worth_execution::ExecutionRequest::serial(&serial);
                let cause = match first.graph_work.prepare_query_read(
                    first.basis.identity(),
                    &request,
                    first.basis.snapshot_handle(),
                    |_| vec![],
                ) {
                    Err(cause) => cause,
                    Ok(_) => panic!("cancelled pin was admitted"),
                };
                assert_eq!(
                    super::super::denial::session(cause, root.entity_id(), &ReadPlan::of(&first)),
                    super::Denial::Cancelled
                );
                return;
            }
            let graph = &world.application.primary_provider.graph;
            let label = AccountLabel::reference();
            let locator = graph
                .layout
                .field_locator(label.entity(), label.aspect(), label.field())
                .unwrap()
                .clone();
            publish_relational_mutation(
                &execution,
                &world,
                WorkerIntentBatch::new("changed-pinned-member").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: root.entity_id(),
                        fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                            locator,
                            StringApplicationValueBinding::encode(&"after-commit".to_owned())
                                .unwrap(),
                        )])),
                    }),
                )),
            );
            let current = world.selected_product(&host);
            let later = current
                .retain_selection(&execution)
                .unwrap()
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    controls(),
                )
                .unwrap();
            assert_ne!(
                first.basis.identity().descriptor(),
                later.basis.identity().descriptor()
            );
            assert_ne!(
                first.basis.snapshot_handle().version_id(),
                later.basis.snapshot_handle().version_id()
            );
            // Unit-owner substitutions isolate otherwise correlated guard inputs.
            // All reads, snapshots and bases here were genuinely issued by these sessions.
            if case == 2 {
                std::sync::Arc::get_mut(&mut pin)
                    .unwrap()
                    .replace_basis_for_test(later.basis.identity().clone());
            } else if case == 3 {
                let fresh = later
                    .graph_work
                    .prepare_query_read(
                        later.basis.identity(),
                        &execution,
                        later.basis.snapshot_handle(),
                        |_| vec![],
                    )
                    .unwrap();
                pin = fresh.retained_pin();
                drop(fresh);
                std::sync::Arc::get_mut(&mut pin)
                    .unwrap()
                    .replace_basis_for_test(first.basis.identity().clone());
            }
            let visits = std::cell::Cell::new(0);
            let rebound = later
                .graph_work
                .prepare_query_read_from_pin(
                    later.basis.identity(),
                    &execution,
                    later.basis.snapshot_handle(),
                    &pin,
                    |_| {
                        visits.set(visits.get() + 1);
                        vec![]
                    },
                )
                .unwrap();
            assert_eq!(visits.get(), 1);
            assert_eq!(
                rebound.retained_pin().basis_for_test(),
                later.basis.identity()
            );
            let (actual, _) = rebound
                .execute(&execution, |native, _| native.snapshot().version_id())
                .unwrap();
            assert_eq!(actual, later.basis.snapshot_handle().version_id());
        })
        .unwrap();
}
