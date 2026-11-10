//! Issuance is independent of deferred index-image residency after recovery.
use super::*;

#[test]
fn residency_cold_index_frontier_prevents_reissue_before_any_image_demand() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for (bulk, admitted) in [(false, false), (true, false), (false, true), (true, true)] {
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
        if admitted {
            let mut total = 0u64;
            let mut funding = execution.reserve_retained_memory(0).unwrap();
            let mut pay = |work: u64, bytes: u64| {
                crate::durability::artifact_payload_emission::request::account_work(
                    execution, work,
                )
                .unwrap();
                total = total.checked_add(bytes).unwrap();
                if let Some(funding) = &mut funding {
                    funding.resize(total).unwrap();
                }
                Ok::<_, ()>(())
            };
            let mut prepared = owner
                .prepare_generation_batch(vec![published.clone()], None, &mut pay)
                .unwrap();
            prepared.admit_actual_geometry(&mut pay).unwrap();
            prepared.publish_admitted();
        } else {
            owner.publish_generation(published.clone());
        }
        assert_eq!(owner.snapshot().generation_frontier(), reserved.0 + 1);
        assert_eq!(fixture.source.reads.load(Ordering::Relaxed), 0);
        owner
            .definition_read_binding()
            .demand_checkpoint_image(execution)
            .unwrap();
        assert!(fixture.source.reads.load(Ordering::Relaxed) > 0);
        let (_, basis) = fixture
            .runtime
            .observe_branch(&fixture.runtime.main_branch_identity())
            .unwrap();
        assert_eq!(
            fixture
                .runtime
                .index_access()
                .read_published_generation_for_observation(
                    execution,
                    fixture.index,
                    &basis.observation()
                )
                .unwrap()
                .unwrap()
                .generation_id,
            reserved
        );
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
