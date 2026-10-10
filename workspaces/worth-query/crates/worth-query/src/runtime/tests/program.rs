use super::support::*;

#[test]
fn compiled_typed_program_installs_runs_and_emits_trace() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let program =
        WorthQueryProgram::compile(FakeDsl, &FakeSchemaAdapter).expect("fake DSL should compile");
    let installed = runtime
        .install_program(program)
        .expect("program should install");
    let operation = installed
        .operation("create_task")
        .expect("operation ref should build");

    let run = runtime
        .run_operation_with_request(
            request_execution,
            operation,
            vec![WorthQueryOperationInput::new(
                "title",
                WorthQueryProgramValue::string("Typed task"),
            )],
        )
        .expect("program should run");
    let trace = runtime.inspect_run(&run).expect("trace should be retained");

    assert_eq!(trace.operation_id(), "create_task");
    assert_eq!(run.outputs()[0].name(), "live:tasks.table");
    let title_value_path = CanonicalFieldPath::new([
        FieldKey::new("title".to_owned()).expect("valid test field"),
        FieldKey::new("value".to_owned()).expect("valid test field"),
    ])
    .expect("valid title value path");
    assert_eq!(
        run.outputs()[0]
            .value()
            .array_field_path_string_value(0, &title_value_path),
        Some("Typed task")
    );
    assert!(trace
        .generated_declarations()
        .iter()
        .any(|declaration| declaration == "live:tasks.table"));
    assert_eq!(trace.write_receipts().len(), 1);
    assert_eq!(trace.patch_artifacts().len(), 1);
    assert!(trace
        .patch_artifacts()
        .iter()
        .any(|artifact| artifact.starts_with("query-delivery:tasks.table:")));
}

#[test]
fn compiled_typed_program_rejects_type_mismatch_before_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let program =
        WorthQueryProgram::compile(FakeDsl, &FakeSchemaAdapter).expect("fake DSL should compile");
    let installed = runtime
        .install_program(program)
        .expect("program should install");
    let operation = installed
        .operation("create_task")
        .expect("operation ref should build");

    let error = runtime
        .run_operation_with_request(
            request_execution,
            operation,
            vec![WorthQueryOperationInput::new(
                "title",
                WorthQueryProgramValue::bool(true),
            )],
        )
        .expect_err("type mismatch should reject before effects execute");

    assert!(matches!(error, WorthQueryRuntimeError::Program(_)));
}
