//! D11: a canonical commit artifact's bytes depend only on durable history.
//! Two independently built memory-only runtimes, which hold different
//! process-local runtime identities, run one script — commits on main, a fork
//! and commits on it, an executed merge and a schema transition — and must
//! produce byte-identical artifacts at every position.

use super::*;

const MAIN: &str = "main";
const FEATURE: &str = "feature";

fn branch(name: &str) -> BranchId {
    BranchId(name.to_string())
}

fn run_script(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
) {
    let entity = script_opening(execution, runtime);
    script_close(execution, runtime, entity);
}

/// Commits on main, a fork and a commit on it.
fn script_opening(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
) -> crate::facade::identity::EntityId {
    let created = create_entity_outcome(execution, runtime, "main-a");
    let entity = created
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::facade::transactions::RecordRef::Entity(entity) => Some(*entity),
            _ => None,
        })
        .expect("the create changes one entity");
    create_branch_from_main(runtime, FEATURE);
    create_entity_outcome_on_branch(execution, runtime, "feature-a", branch(FEATURE));
    create_entity_outcome(execution, runtime, "main-b");
    entity
}

/// An executed merge, a commit on the fork and a schema transition.
fn script_close(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    entity: crate::facade::identity::EntityId,
) {
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: branch(MAIN),
                source_branch: branch(FEATURE),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("the merge prepares");
    runtime
        .execute_prepared_merge(execution, prepared)
        .expect("the merge executes");
    update_entity_on_branch(execution, runtime, entity, "feature-b", branch(FEATURE));
    transition_main_to_v2(execution, runtime);
}

fn transition_main_to_v2(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
) {
    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let input = test_owner_transaction_validation_input_for_main(runtime).with_schema_transition(
        schema_transition_for_subscriber_impact(
            SchemaVersionId(2),
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
        ),
        Some(SchemaReconciliationPolicy::PreserveInformation),
    );
    let mut transaction = runtime
        .begin_branch_transaction(input.basis(), input.intent().clone())
        .expect("owner-admitted transaction");
    transaction
        .push_batch(
            batch_create("main-v2"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the batch stages");
    let transitioned = transaction
        .commit(execution, runtime)
        .expect("the schema transition commits");
    assert!(transitioned.envelope().schema_transition.is_some());
}

fn artifacts(runtime: &RelationalRuntime) -> Vec<Vec<u8>> {
    runtime
        .durability()
        .canonical_commit_artifacts(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .map(|artifact| {
            artifact
                .expect("retained commits encode")
                .as_bytes()
                .to_vec()
        })
        .collect()
}

/// The first line on which two artifacts' readmitted envelopes render
/// differently, to name the field that carries process-local identity when
/// the claim fails.
fn first_difference(left: &[u8], right: &[u8]) -> String {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let render = |bytes: &[u8]| {
        crate::facade::durability::RelationalCanonicalCommitArtifact::readmit(bytes, execution)
            .map(|readmitted| format!("{:#?}", readmitted.envelope()))
            .unwrap_or_else(|refusal| format!("{refusal:?}"))
    };
    let (left, right) = (render(left), render(right));
    let lines = left.lines().collect::<Vec<_>>();
    lines
        .iter()
        .zip(right.lines())
        .position(|(l, r)| *l != r)
        .map(|line| {
            format!(
                "{} > `{}` vs `{}`",
                enclosing_fields(&lines, line),
                lines[line].trim(),
                right.lines().nth(line).unwrap_or_default().trim()
            )
        })
        .unwrap_or_else(|| "renderings agree; encodings differ".to_owned())
}

/// The fields enclosing `line` of a pretty `Debug` rendering, outermost first.
fn enclosing_fields(lines: &[&str], line: usize) -> String {
    let indent = |text: &str| text.len() - text.trim_start().len();
    let mut depth = indent(lines[line]);
    let mut path = Vec::new();
    for text in lines[..line].iter().rev() {
        if indent(text) < depth {
            depth = indent(text);
            path.push(text.trim());
        }
    }
    path.reverse();
    path.join(" > ")
}

#[test]
fn independent_runtimes_publish_byte_identical_canonical_artifacts() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let mut first = runtime_with_test_schema();
    let mut second = runtime_with_test_schema();
    assert_ne!(first.runtime_instance_id(), second.runtime_instance_id());
    run_script(execution, &mut first);
    run_script(execution, &mut second);
    assert_same_artifacts(&first, &second, "independent runtimes");
}

/// A durable runtime that stops partway, is recovered by another process from
/// its medium — from the journal alone, or from a checkpoint and the journal
/// after it — and finishes the script publishes the reference's artifacts.
#[test]
fn a_recovered_runtime_continues_with_the_reference_artifacts() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let mut reference = runtime_with_test_schema();
    run_script(execution, &mut reference);
    for checkpointed in [false, true] {
        let mut stopped = durable_runtime_with_test_schema();
        let entity = script_opening(execution, &mut stopped);
        if checkpointed {
            checkpoint_durably(execution, &stopped).expect("the checkpoint captures");
        }
        let (_, mut recovered) =
            recover_with(execution, &stopped, durable_runtime_with_test_schema);
        drop(stopped);
        script_close(execution, &mut recovered, entity);
        let case = if checkpointed {
            "recovered from a checkpoint"
        } else {
            "recovered from the journal"
        };
        assert_same_artifacts(&reference, &recovered, case);
    }
}

fn assert_same_artifacts(expected: &RelationalRuntime, found: &RelationalRuntime, case: &str) {
    let (left, right) = (artifacts(expected), artifacts(found));
    assert_eq!(left.len(), 6, "{case}: every script commit is retained");
    assert_eq!(left.len(), right.len(), "{case}: commit count");
    for (index, (l, r)) in left.iter().zip(&right).enumerate() {
        assert!(
            l == r,
            "{case}: artifact at position {} differs: {}",
            index + 1,
            first_difference(l, r)
        );
    }
}
