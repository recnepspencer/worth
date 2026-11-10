//! Actual cold selections retain carried indexes and verify postings through admitted Head data.
use super::*;
use crate::indexes::data::BoundedIndexParityMode;

#[test]
fn cold_checkpoint_preserves_carried_index_selection_and_reads_postings_without_a_full_warmup() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for mutable in [false, true] {
        let fixture = super::cold_index_fixture::reopened_carried(execution);
        let (_, basis) = fixture
            .runtime
            .observe_branch(&fixture.runtime.main_branch_identity())
            .unwrap();
        if mutable {
            // The snapshot transport owner has the same deferred-image merge.
            let mut snapshot = fixture.runtime.indexes.snapshot();
            snapshot.demand_checkpoint_image(execution).unwrap();
            fixture.runtime.indexes.install(snapshot);
        }
        let carried = fixture.carried.as_ref().unwrap();
        let access = fixture.runtime.index_access();
        for expected in [&fixture.generation, carried] {
            assert_eq!(
                access
                    .read_published_generation_for_observation(
                        execution,
                        expected.index_id,
                        &basis.observation()
                    )
                    .unwrap()
                    .as_deref(),
                Some(expected.as_ref())
            );
        }
        let view = fixture
            .runtime
            .read_truth()
            .project_observation(&basis.observation())
            .unwrap();
        let lookup = access
            .prepare_entity_field_lookup(
                &view,
                fixture.index,
                KindId(1),
                &aspect_field_locator(aspect_key("name"), field_key("name")),
                execution,
            )
            .unwrap();
        let outcome = lookup
            .execute_prepared_entity_field_lookup(
                &string_aspect_value("matrix-index-updated"),
                2,
                BoundedIndexParityMode::Production,
                execution,
                || Ok::<_, ()>(()),
            )
            .unwrap();
        assert_eq!(outcome.candidate_entity_ids(), &[fixture.entity]);
    }
}
