//! Mutation execution helpers and Directory Home launchers for ordered-index courts.
use super::operations::{CreateTask, DeleteTask, RenameTask, RescheduleTask};
use super::queries::TaskRow;
use super::schema::DaymarkSchema;
use super::{limits, seed_workspace, Home, Host, Program};
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::application_contribution::with_bootstrap_advancement_from_request;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::application_installation::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

pub(super) fn home_lease() -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(128 * 1024 * 1024), None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}

pub(super) fn open_directory_home(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
    initialize: bool,
    initial_tasks: &[(u64, &str, i64, i64)],
) -> Home {
    open_directory_home_with_limits(
        location,
        execution,
        initialize,
        initial_tasks,
        limits(None, None),
    )
}

pub(super) fn open_directory_home_with_limits(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
    initialize: bool,
    initial_tasks: &[(u64, &str, i64, i64)],
    app_limits: WorthQueryApplicationLimits,
) -> Home {
    let declaration = DaymarkSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<DaymarkSchema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let tasks: Vec<(u64, String, i64, i64)> = initial_tasks
        .iter()
        .map(|&(k, t, s, e)| (k, t.to_owned(), s, e))
        .collect();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location.to_path_buf()),
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse("7b7b7b7b-7b7b-7b7b-7b7b-7b7b7b7b7b7b").unwrap(),
        app_limits,
        move |_, graph, installed| {
            assert!(initialize, "reopened Home must not replay initializer");
            let borrowed: Vec<(u64, &str, i64, i64)> = tasks
                .iter()
                .map(|(k, t, s, e)| (*k, t.as_str(), *s, *e))
                .collect();
            seed_workspace(graph, installed, &borrowed)
        },
    );
    with_bootstrap_advancement_from_request(*execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|refused| panic!("directory Home open failed: {refused:?}"))
}

pub(super) fn create_task(
    host: &Host,
    key: u64,
    title: &str,
    start_min: i64,
    end_min: i64,
    idem: u64,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(CreateTask {
            workspace: "main".into(),
            key,
            title: title.to_owned(),
            start_min,
            end_min,
        })
        .without_source()
        .idempotency(&idem)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "create_task outcome: {outcome:?}"
    );
}

pub(super) fn rename_task(host: &Host, key: u64, title: &str, idem: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(RenameTask {
            key,
            title: title.to_owned(),
        })
        .without_source()
        .idempotency(&idem)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "rename_task outcome: {outcome:?}"
    );
}

pub(super) fn reschedule_task(host: &Host, key: u64, start_min: i64, end_min: i64, idem: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(RescheduleTask {
            key,
            start_min,
            end_min,
        })
        .without_source()
        .idempotency(&idem)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "reschedule_task outcome: {outcome:?}"
    );
}

pub(super) fn delete_task(host: &Host, key: u64, idem: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(DeleteTask {
            workspace: "main".into(),
            key,
        })
        .without_source()
        .idempotency(&idem)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "delete_task outcome: {outcome:?}"
    );
}

pub(super) fn keys(rows: &[TaskRow]) -> Vec<u64> {
    rows.iter().map(|r| r.key).collect()
}
