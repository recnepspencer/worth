use super::branch_history_publications::create_entity_outcome_on_branch;
use super::branch_history_publications::fork;
use super::branch_history_publications::update_entity_on_branch;
use super::checkpoint_source_capture::capture_durably;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::merge::MergeExecutionRequest;
use worth_relational::facade::merge::MergeIntent;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::create_entity_outcome;
use worth_testkit_relational::durable_publication::durable_runtime;
use worth_testkit_relational::durable_publication::memory_runtime;

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
) -> worth_relational::facade::identity::EntityId {
    let created = create_entity_outcome(execution, runtime, "main-a");
    let entity = created
        .changed_records
        .iter()
        .find_map(|record| match record {
            worth_relational::facade::transactions::RecordRef::Entity(entity) => Some(*entity),
            worth_relational::facade::transactions::RecordRef::Relation(_) => None,
        })
        .expect("the create changes one entity");
    fork(execution, runtime, FEATURE);
    create_entity_outcome_on_branch(execution, runtime, "feature-a", branch(FEATURE));
    create_entity_outcome(execution, runtime, "main-b");
    entity
}

/// An executed merge, a commit on the fork and a schema transition.
fn script_close(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    entity: worth_relational::facade::identity::EntityId,
) {
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(MergeExecutionRequest::new(
                    branch(MAIN),
                    branch(FEATURE),
                    MergeIntent::ReconcileIntoTarget,
                ))
                .unwrap(),
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
    super::checkpoint_schema_versions::transition_main(execution, runtime, 2, "main-v2");
}

fn artifacts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Vec<Vec<u8>> {
    runtime
        .durability()
        .canonical_commit_artifacts(execution)
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
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let render = |bytes: &[u8]| {
        worth_relational::facade::durability::RelationalCanonicalCommitArtifact::readmit(
            bytes, execution,
        )
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
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let mut first = memory_runtime();
    let mut second = memory_runtime();
    assert_ne!(first.runtime_instance_id(), second.runtime_instance_id());
    run_script(execution, &mut first);
    run_script(execution, &mut second);
    assert_same_artifacts(execution, &first, &second, "independent runtimes");
}

/// A durable runtime that stops partway, is recovered by another process from
/// its medium — from the journal alone, or from a checkpoint and the journal
/// after it — and finishes the script publishes the reference's artifacts.
#[test]
fn a_recovered_runtime_continues_with_the_reference_artifacts() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let mut reference = memory_runtime();
    run_script(execution, &mut reference);
    for checkpointed in [false, true] {
        let medium = TestDurableMedium::new();
        let mut stopped = durable_runtime(&medium);
        let entity = script_opening(execution, &mut stopped);
        if checkpointed {
            capture_durably(execution, &stopped, &medium).expect("the checkpoint captures");
        }
        let (_, mut recovered) =
            super::branch_history_recovery::recover(execution, &medium, durable_runtime(&medium));
        drop(stopped);
        script_close(execution, &mut recovered, entity);
        let case = if checkpointed {
            "recovered from a checkpoint"
        } else {
            "recovered from the journal"
        };
        assert_same_artifacts(execution, &reference, &recovered, case);
    }
}

fn assert_same_artifacts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    expected: &RelationalRuntime,
    found: &RelationalRuntime,
    case: &str,
) {
    let (left, right) = (artifacts(execution, expected), artifacts(execution, found));
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
