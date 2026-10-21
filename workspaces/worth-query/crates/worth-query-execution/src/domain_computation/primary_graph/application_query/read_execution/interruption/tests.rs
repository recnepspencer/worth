use std::{
    cell::{Cell, RefCell},
    num::NonZeroUsize,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::domain_computation::{
    execution_runtime::WorthQueryApplicationQueryResourceProfile,
    primary_graph::{
        tests::fixture::{
            installed_authorization_world_with_resource_profile, status_parameter, AccountStatus,
            NestedAccountQuery,
        },
        WorthQueryApplicationOneShotDenialKind, WorthQueryApplicationQueryAccessContext,
        WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
    },
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

thread_local! {
    // Schedules an external cancellation at a real kernel safe point. It cannot
    // forge admission, substitute rows, or bypass the production meter.
    static CHECKPOINT: RefCell<Option<Box<dyn FnMut(&str)>>> = RefCell::new(None);
}

pub(super) fn visit_checkpoint(subject: &str) {
    CHECKPOINT.with(|slot| {
        if let Some(callback) = slot.borrow_mut().as_mut() {
            callback(subject);
        }
    });
}

struct CheckpointGuard;
impl Drop for CheckpointGuard {
    fn drop(&mut self) {
        CHECKPOINT.with(|slot| *slot.borrow_mut() = None);
    }
}

fn nz(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}

#[test]
fn cancellation_during_real_nested_materialization_releases_all_read_resources() {
    let world = installed_authorization_world_with_resource_profile(
        WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(64).expect("concurrent graph work is non-zero"),
        ),
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let scope = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(NestedAccountQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            selected
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_owned())
                        .unwrap(),
                    WorthQueryProductQueryControls::new(nz(10), &request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let triggered = Rc::new(Cell::new(false));
    let witness = Rc::clone(&triggered);
    CHECKPOINT.with(|slot| {
        *slot.borrow_mut() = Some(Box::new(move |subject| {
            if !witness.get() && subject.contains("relation") {
                witness.set(true);
                cancellation.cancel();
            }
        }))
    });
    let _guard = CheckpointGuard;
    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .err()
                .expect("cancelled read cannot publish rows")
        })
        .expect("the fixture read owns its installed host call");
    assert!(
        triggered.get(),
        "must cancel inside nested traversal at a real nested relation checkpoint"
    );
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::Cancelled
    );

    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        0
    );
}

#[test]
fn deadline_is_checked_at_kernel_safe_points() {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(Some(Instant::now()), cancellation.token());
    let denial = super::checkpoint(&request, "root/relation[0]").unwrap_err();
    assert_eq!(
        denial.kind(),
        super::WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded
    );
}
