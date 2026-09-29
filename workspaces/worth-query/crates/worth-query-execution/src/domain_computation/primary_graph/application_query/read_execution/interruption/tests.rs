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
        WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryPrincipalResolutionMode,
        WorthQueryProductQueryControls,
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
        WorthQueryApplicationQueryResourceProfile::bounded(5120, 65_536, 100_000, 64).unwrap(),
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(NestedAccountQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = selected
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), "open".to_owned())
                .unwrap(),
            WorthQueryProductQueryControls::new(nz(10), nz(10_000), &request),
        )
        .unwrap();
    let buffers = world.application.result_buffer_observer();
    let observed = buffers.clone();
    let triggered = Rc::new(Cell::new(false));
    let witness = Rc::clone(&triggered);
    CHECKPOINT.with(|slot| {
        *slot.borrow_mut() = Some(Box::new(move |subject| {
            if !witness.get()
                && subject.contains("relation")
                && observed.observe().retained_bytes() > 0
            {
                witness.set(true);
                cancellation.cancel();
            }
        }))
    });
    let _guard = CheckpointGuard;
    let denial = world
        .application
        .execute_application_query_one_shot(plan)
        .err()
        .expect("cancelled read cannot publish rows");
    assert!(
        triggered.get(),
        "must cancel inside nested traversal after real buffer claims"
    );
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::Cancelled
    );
    assert!(buffers.observe().peak_observed_bytes() > 0);
    assert_eq!(buffers.observe().active_buffers(), 0);
    assert_eq!(buffers.observe().retained_bytes(), 0);
    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
    assert_eq!(world.application.provider_session_resource_count(), 0);
}

#[test]
fn direct_query_admission_cannot_bypass_host_work_policy() {
    let world = installed_authorization_world_with_resource_profile(
        WorthQueryApplicationQueryResourceProfile::bounded(5120, 65_536, 100_000, 64)
            .unwrap()
            .with_maximum_work(nz(100)),
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(NestedAccountQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let denial = selected
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), "open".to_owned())
                .unwrap(),
            WorthQueryProductQueryControls::new(nz(10), nz(101), &request),
        )
        .err()
        .expect("lower admission must enforce the host guard without front-door help");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded
    );
    assert_eq!(
        world
            .application
            .result_buffer_observer()
            .observe()
            .active_buffers(),
        0
    );
    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
    assert_eq!(world.application.provider_session_resource_count(), 0);
}

#[test]
fn deadline_is_checked_at_kernel_safe_points() {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(Instant::now(), cancellation.token());
    let denial = super::checkpoint(&request, "root/relation[0]").unwrap_err();
    assert_eq!(
        denial.kind(),
        super::WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded
    );
}
