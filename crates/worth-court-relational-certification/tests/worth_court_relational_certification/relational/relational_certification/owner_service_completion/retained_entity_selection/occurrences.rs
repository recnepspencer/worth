//! Equal values retain distinct historical provenance and original publication guards.
use super::*;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis, mvcc::PreparedRelationalCommitCandidate,
};

#[test]
fn equal_payload_selections_preserve_source_occurrences_and_cannot_overwrite_a_winner() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let original = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let changed = update(execution, &runtime, [(model, "5")]);
    let returned = update(execution, &runtime, [(model, "3")]);
    let aba = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let winner = prepare(execution, &runtime, &original, model);
    let loser = prepare(execution, &runtime, &aba, model);
    assert_eq!(winner.prepared_changed_record_count(), Some(1));
    let first = perform(execution, &runtime, winner);
    assert!(matches!(
        runtime
            .owner_component_services()
            .publication_port()
            .compare_and_publish(execution, loser),
        RelationalPublicationOutcome::Stale(_)
    ));
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id()
            .unwrap(),
        first.commit.commit_id
    );
    let second = perform(
        execution,
        &runtime,
        prepare(execution, &runtime, &aba, model),
    );
    assert_eq!(second.commit.parents, vec![first.commit.commit_id]);
    assert_ne!(first.commit.commit_id, second.commit.commit_id);
    assert_eq!(
        historical_commit(execution, &runtime, &first),
        old.commit.commit_id.0
    );
    assert_eq!(
        historical_commit(execution, &runtime, &second),
        returned.commit.commit_id.0
    );
    for commit in [&old, &returned, &first, &second] {
        assert_eq!(
            values(execution, &runtime, commit, [model, model]),
            ["3", "3"].map(string_aspect_value)
        );
    }
    for commit in [&old, &changed, &returned, &first, &second] {
        release_test_commit_snapshot(&runtime, commit);
    }
}

#[test]
fn selection_on_a_current_fork_preserves_the_original_future_and_exact_cross_branch_source() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let historical = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let current = update(execution, &runtime, [(model, "5")]);
    let unchanged_main = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let services = runtime.owner_component_services();
    let (_, source) = services
        .fork_port()
        .observe_fork_source(&worth_relational::facade::history::BranchId("main".into()))
        .unwrap();
    let fork = services
        .fork_port()
        .fork_branch(
            execution,
            worth_relational::facade::history::BranchId("selected-future".into()),
            source,
        )
        .unwrap();
    let child = runtime.admit_branch_basis(fork.target_identity()).unwrap();
    let transaction = runtime
        .begin_branch_transaction(&child, RelationalTransactionIntent::ordinary())
        .unwrap();
    let selection = services
        .preparation_port()
        .admit_retained_entity_selection(transaction, &historical, model)
        .unwrap();
    let candidate = services
        .preparation_port()
        .prepare_retained_entity_selection(execution, selection)
        .unwrap();
    let selected = perform(execution, &runtime, candidate);
    assert_eq!(selected.commit.parents, vec![current.commit.commit_id]);
    assert_eq!(
        selected.commit.branch_id,
        child.identity().branch_id().clone()
    );
    assert_eq!(
        historical_commit(execution, &runtime, &selected),
        old.commit.commit_id.0
    );
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .reference(),
        unchanged_main.reference()
    );
    assert_eq!(
        values(execution, &runtime, &selected, [model, model]),
        ["3", "3"].map(string_aspect_value)
    );
    assert_eq!(
        values(execution, &runtime, &current, [model, model]),
        ["5", "5"].map(string_aspect_value)
    );
    for commit in [&old, &current, &selected] {
        release_test_commit_snapshot(&runtime, commit);
    }
}

pub(crate) fn fixture() -> AspectSchemaFixture {
    AspectSchemaFixture {
        entity_aspects: vec![entity_field_aspect(aspect_key("name"), field_key("label"))],
        ..AspectSchemaFixture::default()
    }
}

pub(super) fn prepare(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: &AdmittedRelationalBranchBasis,
    model: EntityId,
) -> PreparedRelationalCommitCandidate {
    let current = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let transaction = runtime
        .begin_branch_transaction(&current, RelationalTransactionIntent::ordinary())
        .unwrap();
    let services = runtime.owner_component_services();
    let input = services
        .preparation_port()
        .admit_retained_entity_selection(transaction, source, model)
        .unwrap();
    services
        .preparation_port()
        .prepare_retained_entity_selection(execution, input)
        .unwrap()
}

pub(crate) fn perform(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    candidate: PreparedRelationalCommitCandidate,
) -> CommitResult {
    let services = runtime.owner_component_services();
    let RelationalPublicationOutcome::Performed(performed) = services
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("publication must perform")
    };
    services
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap()
}

fn historical_commit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit: &CommitResult,
) -> u64 {
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(ExecutionAllocationPolicy::SystemAllocation)
        .map(Result::unwrap)
        .find(|artifact| artifact.commit_id() == commit.commit.commit_id)
        .unwrap();
    let admitted =
        RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), execution).unwrap();
    admitted
        .envelope()
        .authority_kind()
        .retained_entity_selection()
        .unwrap()
        .source_head()
        .selected_commit_id()
}
