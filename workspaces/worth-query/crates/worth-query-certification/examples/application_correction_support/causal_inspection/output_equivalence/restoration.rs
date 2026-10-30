//! Actual current-format restoration recovers output authority, never an in-memory comparison.
use super::super::super::{
    authentication::*,
    edit::{Edit, Value},
    host,
    output_program::{OutputProgram, Root},
    program,
    schema::NumericSchema,
};
use super::super::{distinct_subjects::observation, journey as source_journey};
use super::{
    demand,
    journey::{inspect, install},
};
use worth_query_host::facade::{
    application_entry::*, application_installation::*, causal_inspection::*,
    domain::ApplicationSchema, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

fn restore(checkpoint: WorthQueryApplicationCheckpoint) -> host::OutputHost {
    in_memory_rostered_program_from_checkpoint(
        program::validated::<OutputProgram>(),
        WorthQueryApplicationProgramRoster::new().support(program::validated::<program::Program>()),
        NumericSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        host::installation_limits(),
        checkpoint,
    )
    .expect("the same program/roster/build/limits restore without a seed callback")
}

pub fn restored_output_has_no_original_in_memory_evaluation() {
    let original = install();
    let (old_source, _, read) =
        source_journey::perform(&original, 4, "restored-evaluation-source-four");
    drop(read);
    let (old_producer, read, contacts) = demand::settle(&original);
    assert_eq!(contacts, 1);
    drop(read);
    let checkpoint = original
        .with_host_advancement(|phase| {
            original
                .runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    assert!(!checkpoint.bytes().is_empty());
    drop((old_source, old_producer, original));
    let application = restore(checkpoint);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let current = request
        .query(super::super::super::query::Read {
            key: "numeric".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        (current.rows()[0].primary, current.rows()[0].secondary),
        (4, 7),
        "actual checkpoint truth replaces the bootstrap seed"
    );
    drop(current);
    // A fresh actual no-op source receipt belongs to this restored runtime.
    // Equal Native facts allow restoration; the foreign pre-restart receipt is never substituted.
    let performed = request
        .advanced()
        .mutate(Edit {
            key: "numeric".to_owned(),
            value: Value::Primary(4),
        })
        .without_source()
        .idempotency(&"restored-evaluation-fresh-noop-source".to_owned())
        .execute_performed::<OutputProgram, Root>(
            &application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
        panic!("the restored runtime must issue its own genuine performed source")
    };
    let (source, _, read) = source_journey::settle_source(&application, &request, performed, 0);
    drop(read);
    let (receipt, read, contacts, posture) = demand::settle_observation(&application);
    assert_eq!(
        posture,
        WorthQueryOutputSettlementPosture::RecoveredPerformed
    );
    assert!(
        receipt.is_none(),
        "restored authority cannot forge an original live receipt"
    );
    assert_eq!(
        contacts, 0,
        "checkpoint restoration executes no parity handler"
    );
    let (original, row) =
        observation::read(&application, &principal, &scope, &read, "observer-decoy");
    assert_eq!(row.references, vec![("observer".to_owned(), 0)]);
    drop(read);
    let denial = inspect(&application, &principal, &scope, &original, &source)
        .err()
        .expect("restoration cannot recreate an original in-memory comparison");
    assert_eq!(
        denial.producer_evaluation_refusal(),
        Some(WorthQueryProducerEvaluationDenial::NotPerformed)
    );
    drop(denial);
    original.close();
    assert_eq!(readers.observe().active(), baseline);
}
