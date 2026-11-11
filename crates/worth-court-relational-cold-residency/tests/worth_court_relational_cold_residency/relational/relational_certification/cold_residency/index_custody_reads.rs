//! An original index directory retains its actual external frame through replacement.
use super::index_source::IndexRecipe;
use worth_testkit_relational::durable_medium::member_placement::ReplacingMemberSource;
#[test]
fn an_older_index_directory_retains_its_cold_frame_after_home_replacement() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let recipe = IndexRecipe::new(request, false);
    let mut placed = None;
    let fixture = recipe.reopen_with_source(request, |producer| {
        let source = ReplacingMemberSource::new(producer);
        placed = Some(source.clone());
        source
    });
    let source = placed.unwrap();
    source.replace_selected_placement();
    let (_, basis) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let view = fixture
        .runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    let pin = fixture
        .runtime
        .read_truth()
        .pin_view_read_capability(&view, &[fixture.index], request)
        .unwrap();
    assert_eq!(
        pin.index_generation(fixture.index),
        Some(fixture.generation.generation_id)
    );
    assert!(source
        .producer
        .member_reads
        .lock()
        .unwrap()
        .contains(&fixture.member));
    drop(pin);
    drop(view);
    drop(basis);
    drop(fixture);
    assert!(
        source.owners_released(),
        "the final cold image owner releases actual placement"
    );
}
