use super::super::support::*;

#[test]
fn aspect_native_mutation_builders_reject_empty_or_duplicate_authoring() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.aspect-errors")
        .expect("task runtime should open a named workspace");

    let empty = workspace
        .insert(request_execution, "Task", |task| task)
        .expect_err("empty aspect mutation should fail closed");
    match empty {
        WorthQueryRuntimeError::Workspace(error) => {
            assert!(error.to_string().contains("at least one aspect"));
        }
        other => panic!("expected workspace authoring error, got {other:?}"),
    }

    let duplicate = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Buy milk"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Buy oat milk"),
            )
        })
        .expect_err("duplicate aspect paths should fail closed");
    match duplicate {
        WorthQueryRuntimeError::Workspace(error) => {
            assert!(error.to_string().contains("may only be declared once"));
        }
        other => panic!("expected workspace authoring error, got {other:?}"),
    }

    let duplicate_clear = workspace
        .update(
            request_execution,
            test_entity_identity("entity:1:1:1"),
            |task| {
                task.clear(test_aspect_touch("title.value")).set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Buy milk"),
                )
            },
        )
        .expect_err("clear and set of the same aspect should fail closed");
    match duplicate_clear {
        WorthQueryRuntimeError::Workspace(error) => {
            assert!(error.to_string().contains("may only be declared once"));
        }
        other => panic!("expected workspace authoring error, got {other:?}"),
    }

    let empty_batch = workspace
        .batch(request_execution, |batch| batch)
        .expect_err("empty mutation batch should fail closed");
    match empty_batch {
        WorthQueryRuntimeError::Workspace(error) => {
            assert!(error.to_string().contains("at least one operation"));
        }
        other => panic!("expected workspace authoring error, got {other:?}"),
    }
}
