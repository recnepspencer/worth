//! A deferred root freezes unrelated work until its owner delivers or closes.
use super::*;
use std::{collections::BTreeMap, sync::Arc};
use worth_query_host::facade::application_contribution::WorthQueryApplicationContributionTuple;
use worth_query_host::facade::primary_graph::{
    advancement_observations_on_this_thread_for_test as observations,
    WorthQueryAdvancementEventKindForTest as Kind,
    WorthQueryAdvancementIdentityForTest as Identity,
};

struct RestoreSchedule;
impl Drop for RestoreSchedule {
    fn drop(&mut self) {
        computation::arm(None);
    }
}

fn case<P>(deferred_rank: usize, stale: bool)
where
    P: ApplicationProgramDefinition<
        CheckpointSchema,
        Outputs = ApplicationProgramOutputs<program::ChainRoot>,
    >,
    P::Contributions: WorthQueryApplicationContributionTuple<
        CheckpointSchema,
        Configuration = (TopologyConfiguration,),
    >,
{
    let _restore = RestoreSchedule;
    drop(observations());
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = Arc::new(support::install_program_with_seed::<P>(
        None,
        profile,
        4096,
        128 * 1024 * 1024,
        // This fixture explicitly bounds invalidation independently of source proof.
        Some(1_000_000),
        |graph| {
            diamond::seed(graph);
            ring_world::seed::<1>(graph);
        },
    ));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    macro_rules! root {
        ($key:expr) => {
            request
                .demand(PlanarOutputDemand::new($key))
                .start_in_program::<P, program::ChainRoot>(&application)
                .unwrap()
        };
    }
    let mut left = root!("diamond-left");
    let mut right = root!("diamond-right");
    let mut d = root!("ring-0-a");
    let mut unrelated = root!("ring-0-source-b");
    let initial = unrelated.advance(&request).unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(_) = initial else {
        panic!("unrelated caller is Current before deferral")
    };
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (0, 0)
    );
    let mut caller = request
        .demand(ChainDemand("diamond-join".into()))
        .start_dependent_in_program::<P, program::ChainConnection>(&application)
        .unwrap();
    for demand in [&mut left, &mut right, &mut d] {
        assert!(matches!(
            demand.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
    }
    assert!(matches!(
        caller.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let mut names = BTreeMap::new();
    for key in [
        "diamond-left",
        "diamond-right",
        "ring-0-a",
        "ring-0-source-b",
    ] {
        let selected = request
            .query(PlanarRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        names.insert(
            selected.observed_sources()[0].root_entity_for_test(),
            key.to_owned(),
        );
    }
    let d_root = *names
        .iter()
        .find(|(_, name)| name.as_str() == "ring-0-a")
        .unwrap()
        .0;
    let mut keys: Vec<_> = observations()
        .into_iter()
        .flat_map(|record| record.applied)
        .filter_map(|event| match event.identity {
            Identity::Producer {
                producer,
                query,
                parameters,
                root,
                ..
            } if event.kind == Kind::Commit
                && names
                    .get(&root)
                    .is_some_and(|name| name.starts_with("diamond-")) =>
            {
                Some(((producer, query, parameters, root), names[&root].clone()))
            }
            _ => None,
        })
        .collect();
    keys.sort();
    assert_eq!(keys.len(), 2);
    let roots: [String; 2] = keys
        .into_iter()
        .map(|(_, name)| name)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let own_root = *names
        .iter()
        .find(|(_, name)| name.as_str() == "ring-0-source-b")
        .unwrap()
        .0;
    let live_contacts_before =
        application.producer_contacts_at_root_on_this_thread_for_test(own_root);
    let WorthQueryApplicationOutputDemandProgress::Settled(live) =
        unrelated.advance(&request).unwrap()
    else {
        panic!("the Live control certifies its unchanged caller as Current");
    };
    assert_eq!(
        application.entered_required_selection_is_frozen_for_test(),
        Some(false)
    );
    let live_contacts = application.producer_contacts_at_root_on_this_thread_for_test(own_root)
        - live_contacts_before;
    assert_eq!(
        live_contacts, 0,
        "the Live control needs no handler execution"
    );
    for (rank, key) in roots
        .iter()
        .chain(std::iter::once(&"ring-0-a".to_owned()))
        .enumerate()
    {
        let selected = request
            .query(PlanarRead {
                body_key: key.clone(),
            })
            .execute()
            .unwrap();
        let y = match key.as_str() {
            "diamond-left" => 5,
            "diamond-right" => 54,
            "ring-0-a" => 2,
            _ => panic!("the seeded root is declared"),
        };
        request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: key.clone(),
                replacement_y: length(y),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&(0x9176_7800_u64 + rank as u64))
            .execute_performed::<P, program::ChainRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
    }
    let schedule = computation::Schedule::new(roots, None, false);
    let fault_owner = Arc::clone(&application);
    *schedule.owner_completion.lock().unwrap() = Some(Arc::new(move |rank| {
        if rank == deferred_rank {
            fault_owner.delay_next_output_readiness_delivery_for_test();
        }
    }));
    let queued = application.queued_required_rows_for_root_for_test(d_root);
    assert!(!queued.is_empty(), "the seeded edit queues d's issued row");
    computation::arm(Some(Arc::clone(&schedule)));
    let before = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    let result = caller.advance(&request);
    let after = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        schedule.state.lock().unwrap().completed,
        [0, 1],
        "both roots reached owner completion in canonical order"
    );
    if stale {
        assert!(
            matches!(result,
            Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(ref stop))
                if stop.kind() == primary_graph::WorthQueryOutputDemandDenialKind::PublicationStale),
            "the owner's ordinary sibling read refuses the stale caller plan"
        );
    } else {
        assert!(
            matches!(
                result,
                Ok(WorthQueryApplicationOutputDemandProgress::Pending)
            ),
            "the caller waits for the deferred root"
        );
    }
    assert_eq!(
        after.get(&d_root).copied().unwrap_or(0) - before.get(&d_root).copied().unwrap_or(0),
        0,
        "deferral rank {deferred_rank}: no unrelated root is inspected on the moved head"
    );
    assert_eq!(
        application.queued_required_rows_for_root_for_test(d_root),
        queued,
        "d's exact issued rows remain queued, not merely some row"
    );
    computation::arm(None);
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (1, 1)
    );
    let frozen_reads = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    let own_contacts = application.producer_contacts_at_root_on_this_thread_for_test(own_root);
    let frozen = unrelated.advance(&request).unwrap();
    assert_eq!(
        application.entered_required_selection_is_frozen_for_test(),
        Some(true)
    );
    let WorthQueryApplicationOutputDemandProgress::Settled(frozen) = frozen else {
        panic!("native Current inspection preserves the unrelated caller's result")
    };
    assert_eq!(
        frozen.producer_contacts_in_this_demand(),
        live.producer_contacts_in_this_demand()
    );
    assert_eq!(
        application.producer_contacts_at_root_on_this_thread_for_test(own_root) - own_contacts,
        live_contacts
    );
    assert_eq!(frozen.posture(), live.posture());
    assert_eq!(
        frozen
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        live.outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id()
    );
    assert_eq!(
        primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test(),
        frozen_reads,
        "a frozen entry reads no source, on the caller or any other row"
    );
    assert_eq!(
        application.queued_required_rows_for_root_for_test(d_root),
        queued
    );
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (1, 1)
    );
    // This consumer actually requires the deferred upstream. Its successor
    // is in registry frame custody; native certification lends the sole thaw.
    assert!(matches!(
        caller.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    assert_eq!(
        application.entered_required_selection_is_frozen_for_test(),
        Some(true)
    );
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (0, 0)
    );
    let later = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        later.get(&d_root).copied().unwrap_or(0) - after.get(&d_root).copied().unwrap_or(0),
        1
    );
    assert!(matches!(
        unrelated.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let final_reads = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        final_reads.get(&d_root),
        later.get(&d_root),
        "the next different caller does not read d again"
    );
    assert!(application
        .queued_required_rows_for_root_for_test(d_root)
        .is_empty());
}

#[test]
fn a_deferral_freezes_selection_in_both_commit_orders() {
    let _guard = checkpoint_recovery_test_guard();
    case::<application::Program>(0, false);
    case::<application::Program>(1, false);
}

#[test]
fn a_stale_credit_after_deferral_leaves_the_unrelated_row_queued() {
    let _guard = checkpoint_recovery_test_guard();
    case::<ordinary_read_application::Program>(0, true);
}
