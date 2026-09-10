use std::num::NonZeroUsize;

use crate::query_probe::{assert_ordinary_work, read};
use crate::world::CourtroomWorld;
use worth_query_host::facade::primary_graph;

pub(crate) fn assert_retained_partial_slopes() {
    let mut baseline = None;
    for size in [1, 8, 64] {
        let world = CourtroomWorld::publish("ready");
        let root = world.application.current_world();
        let branches = (0..size)
            .map(|_| {
                world
                    .application
                    .branches()
                    .fork(root)
                    .components(|components| components.fork_relational().fork_signal())
                    .create()
                    .expect("each partial-producing branch must be created")
            })
            .collect::<Vec<_>>();
        let integration = world
            .application
            .granular_invalidation_installation()
            .retain_primary_graph_integration_handle();
        let mut recoveries = Vec::with_capacity(size);
        for (ordinal, branch) in branches.iter().copied().enumerate() {
            integration
                .execute_mutation_with_index_refresh(|runtime| {
                    runtime.fail_next_durable_append_for_test();
                    Ok::<(), ()>(())
                })
                .unwrap()
                .unwrap();
            let outcome = world.change_input_on_branch_with_ordinal(
                branch,
                &format!("retained-partial-{ordinal}"),
                ordinal as u8 + 1,
            );
            let primary_graph::WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) =
                outcome
            else {
                panic!("injected owner durability failure must retain a typed partial")
            };
            assert!(partial.relational_requires_settlement());
            let recovery = partial.into_recovery();
            recovery
                .continue_owner_settlement()
                .expect("owner settlement must complete without publishing product truth");
            assert!(!recovery.inspect().unwrap().relational_requires_settlement());
            recoveries.push(recovery);
        }

        let page = world
            .application
            .product_publication_recovery_page(None, NonZeroUsize::new(size).unwrap())
            .unwrap();
        let actual = page.rows().len();
        eprintln!(
            "axis=RetainedPartials requested={size} actual={actual} examined={} covariation={size} independent product/component branches and settled owner effects",
            page.examined()
        );
        assert_eq!(actual, size);
        let work = read(&world, root).work;
        assert_ordinary_work(work);
        assert_eq!(*baseline.get_or_insert(work), work, "P={size}");
        drop(page);

        for recovery in recoveries {
            let cleanup = world
                .application
                .release_product_publication_recovery(recovery, 0)
                .expect("a settled retained partial must release through the public facade");
            assert_eq!(cleanup.retired_component_count(), 0);
        }
        assert!(world
            .application
            .product_publication_recovery_page(None, NonZeroUsize::new(size).unwrap())
            .unwrap()
            .rows()
            .is_empty());
        for branch in branches {
            let cleanup = world
                .application
                .on_branch(branch)
                .close()
                .expect("recovery cleanup must reclaim the exact unpublished history child");
            assert!(cleanup.is_complete());
        }
    }
}
