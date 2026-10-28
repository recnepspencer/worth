//! Issuance is independent of deferred index-image residency after recovery.
use super::*;

#[test]
fn residency_cold_index_frontier_prevents_reissue_before_any_image_demand() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for bulk in [false, true] {
        let fixture = super::cold_index_fixture::reopened(execution);
        let owner = &fixture.runtime.indexes;
        let reserved = if bulk {
            owner.reserve_generation_ids(1).unwrap()[0]
        } else {
            crate::indexes::data::DerivedIndexGenerationId(owner.next_generation_id().unwrap())
        };
        assert!(reserved.0 > fixture.generation.generation_id.0);
        assert_eq!(fixture.source.reads.load(Ordering::Relaxed), 0);
        let mut published = fixture.generation.as_ref().clone();
        published.generation_id = reserved;
        owner.publish_generation(published.clone());
        assert_eq!(fixture.source.reads.load(Ordering::Relaxed), 0);
        owner
            .definition_read_binding()
            .demand_checkpoint_image(execution)
            .unwrap();
        assert!(fixture.source.reads.load(Ordering::Relaxed) > 0);
        let loaded = owner.all_generations();
        assert_eq!(
            loaded
                .iter()
                .find(|generation| generation.generation_id == fixture.generation.generation_id)
                .map(Arc::as_ref),
            Some(fixture.generation.as_ref())
        );
        assert_eq!(
            loaded
                .iter()
                .find(|generation| generation.generation_id == reserved)
                .map(Arc::as_ref),
            Some(&published)
        );
    }
}
