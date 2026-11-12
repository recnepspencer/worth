//! The actual edit checkpoint reopens from disk with exact source and output rows.
use super::super::super::{calculation, program};
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use worth_query_host::facade::{application_installation::*, domain::ApplicationSchema};
static NEXT: AtomicU64 = AtomicU64::new(0);

pub(super) fn assert_reopened_ready(application: &host::OutputHost, expected: u64) {
    let rows = host::read(application);
    let checkpoint = application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let path = std::env::temp_dir().join(format!(
        "worth-paired-ordinary-{}-{}.checkpoint",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let origin = checkpoint
        .capture_origin()
        .expect("actual capture issued Native and World origin");
    std::fs::write(&path, checkpoint.bytes()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    let restored_checkpoint = application
        .with_host_advancement(|phase| {
            let execution = phase
                .checked_execution_request_for(application.runtime().product_runtime())
                .unwrap();
            WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes.into_boxed_slice())
                .with_capture_origin(origin, &execution)
        })
        .unwrap()
        .expect("disk bytes retain their actual captured Native and World origin");
    let cold: host::OutputHost = WorthQueryInterimCheckpointRestore::rostered_program(
        program::validated::<OutputProgram>(),
        host::roster().support(program::validated::<program::Program>()),
        NumericSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        host::installation_limits(),
        restored_checkpoint,
    )
    .expect("ordinary edit reopens from actual disk bytes without a new origin");
    assert_eq!(host::read(&cold), rows);
    assert_eq!(read_calculation(&cold), expected);
    let scope = request_scope();
    let principal = authenticate_operator(cold.installed_schema(), &scope);
    let request = cold.request(&principal, &scope);
    let before = head(&cold);
    let ((), writes) = observe_maintenance_work(8, || {
        let mut demand = request
            .demand(calculation::Demand {
                key: "numeric".into(),
            })
            .start_in_program::<OutputProgram, Root>(&cold)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.settle(&request).unwrap()
        else {
            panic!("restored ordinary output must settle")
        };
        assert_eq!(
            settled.producer_contacts_in_this_demand(),
            0,
            "cold readiness executes no producer"
        );
    });
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(head(&cold), before);
    assert_eq!(host::read(&cold), rows);
    assert_eq!(read_calculation(&cold), expected);
    assert_next_edit(&cold);
}

// A cold ordinary lineage must remain an ordinary writer on the next real edit.
fn assert_next_edit(application: &host::OutputHost) {
    use super::super::super::edit::{Edit, Value};
    use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.request(&principal, &scope);
    let before = application
        .runtime()
        .producer_contacts_on_this_thread_for_test();
    let (outcome, commits) = observe_maintenance_work(8, || {
        request
            .advanced()
            .mutate(Edit {
                key: "numeric".into(),
                value: Value::Primary(17),
            })
            .without_source()
            .idempotency(&"cold-next-ordinary-edit".to_owned())
            .execute_performed::<OutputProgram, Root>(
                application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the next cold edit remains a genuine ordinary coupled writer")
    };
    assert!(performed.receipt().aftermath_causality().is_none());
    assert_eq!(read_calculation(application), 35);
    assert_eq!(
        commits.published_rows.len() + commits.omitted_publications,
        1
    );
    assert_eq!(
        application
            .runtime()
            .producer_contacts_on_this_thread_for_test()
            - before,
        1
    );
}
