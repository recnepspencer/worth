//! own retirement preserves each read locator and checkpoint currentness.
use super::*;

#[test]
fn own_retirement_preserves_each_read_locator_and_checkpoint_currentness() {
    let world = installed_authorization_world(true);
    world.application.with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
let host_custody = phase.test_host_request();

let (entity, kind, facts) = reads(
            &host_custody.phase_for_application(&world.application),
            execution,
            &world,
            "account-2",
        );
        let locators = facts.iter().map(Fact::locator_identity).collect::<Vec<_>>();
        assert!(
            rebase_current(
                &host_custody.phase_for_application(&world.application),
                &host_custody.borrowed_request_for_test(),
                &world,
                facts.clone(),
                &BTreeSet::from([entity])
            )
            .is_none(),
            "a selected retirement cannot replace reads while its actual record is live"
        );
        delete(&host_custody.borrowed_request_for_test(), &world, entity);
        let retired = rebase_current(
            &host_custody.phase_for_application(&world.application),
            &host_custody.borrowed_request_for_test(),
            &world,
            facts.clone(),
            &BTreeSet::from([entity]),
        )
        .unwrap();
        assert_eq!(
            retired.len(),
            facts.len(),
            "every tracked read remains accounted for"
        );
        assert_eq!(
            retired
                .iter()
                .map(Fact::locator_identity)
                .collect::<Vec<_>>(),
            locators
        );
        assert!(retired.iter().all(|fact| matches!(fact, Fact::RetiredOutputEntity { entity_id, kind: actual, .. } if *entity_id == entity && *actual == kind)));
        assert!(selection(
            &host_custody.phase_for_application(&world.application),
            execution,
            &world,
            &retired
        ));
        let encoded =
        crate::domain_computation::primary_graph::application_checkpoint::encode_producer_facts(
            &retired,
        )
        .unwrap();
        let restored =
        crate::domain_computation::primary_graph::application_checkpoint::decode_producer_facts(
            &encoded,
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        assert_eq!(restored.iter().cloned().collect::<Vec<_>>(), retired.as_slice());
        assert!(selection(
            &host_custody.phase_for_application(&world.application),
            execution,
            &world,
            &restored
        ));
        let external = rebase_current(
            &host_custody.phase_for_application(&world.application),
            &host_custody.borrowed_request_for_test(),
            &world,
            facts,
            &BTreeSet::new(),
        )
        .unwrap();
        assert!(!matches!(external[0], Fact::RetiredOutputEntity { .. }));
        assert!(
            !selection(
                &host_custody.phase_for_application(&world.application),
                execution,
                &world,
                &external
            ),
            "an external input deleted without an owned retirement remains stale"
        );
        let Fact::RetiredOutputEntity {
            created_at,
            deleted_at,
            read_locator,
            ..
        } = &retired[0]
        else {
            unreachable!()
        };
        for (changed_entity, changed_kind, changed_creation) in [
            (
                EntityId::new(
                    entity.partition_id,
                    entity.local_slot_value(),
                    entity.generation_value() + 1,
                ),
                kind,
                *created_at,
            ),
            (entity, KindId::new(kind.as_u32() + 1), *created_at),
            (
                entity,
                kind,
                worth_relational::facade::identity::VersionId(created_at.as_u64() + 1),
            ),
            (
                reads(
                    &host_custody.phase_for_application(&world.application),
                    execution,
                    &world,
                    "account-1",
                )
                .0,
                kind,
                *created_at,
            ),
        ] {
            assert!(!selection(
                &host_custody.phase_for_application(&world.application),
                execution,
                &world,
                &[Fact::RetiredOutputEntity {
                    entity_id: changed_entity,
                    kind: changed_kind,
                    created_at: changed_creation,
                    deleted_at: *deleted_at,
                    read_locator: read_locator.clone(),
                }]
            ));
        }
        let mut changed_deletion = retired[0].clone();
        if let Fact::RetiredOutputEntity { deleted_at, .. } = &mut changed_deletion {
            *deleted_at = worth_relational::facade::identity::VersionId(deleted_at.as_u64() + 1);
        }
        assert!(!selection(
            &host_custody.phase_for_application(&world.application),
            execution,
            &world,
            &[changed_deletion]
        ));

}).expect("the fixture block owns its installed host call")
}
