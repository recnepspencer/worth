//! An item routed again into a partition identity that another key digest
//! holds makes the run again in full. The retained routing is made to hold
//! a kept item under a digest that shares the moved item's partition identity
//! and is not its digest, which no real run retains: a fresh build of the
//! same items meets no collision, and the run made again equals it, charging
//! the reader what it charges.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_operation::application_computation_partition_identity;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::routing::ComputationPartitionRouting;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::RetainedPartitions;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::reroute::moved_facts;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::reroute::owner::Rerouted;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;

/// The key digest of `key`.
pub fn digest_of(key: &Parity) -> [u8; 32] {
    let identity = application_computation_partition_identity(key, &mut |_| Ok::<(), ()>(()));
    *identity.expect("the key encodes").digest()
}

/// The prior a producer hands its next run from `sealed`, its routing
/// holding `kept` under a digest that shares `key`'s partition identity and
/// is not `key`'s digest. The items live outside the facts, so the
/// membership's fact is moved for the run to plan them again.
pub fn colliding(
    mut sealed: SealedComputationRun,
    kept: PartitionItemId,
    key: &Parity,
) -> ComputationPrior {
    let typed = std::mem::replace(&mut (*(*sealed.state_mut_for_test()).typed_mut_for_test()), Arc::new(()))
        .downcast::<RetainedPartitions<Parity, Number, u64>>()
        .expect("the run retains its partitions");
    let mut typed = Arc::try_unwrap(typed)
        .ok()
        .expect("the sealed run holds its partitions alone");
    let mut foreign = digest_of(key);
    foreign[31] ^= 1;
    let mut routing = ComputationPartitionRouting::default();
    for (identity, partition) in (*typed.partitions_for_test()).iter_for_test() {
        for item in (*typed.routing_for_test()).members_for_test(*identity) {
            let digest = if item == kept {
                foreign
            } else {
                digest_of(&(*partition.key_for_test()))
            };
            routing
                .route_for_test(item, digest, |_| Ok(()))
                .expect("the item routes");
        }
    }
    (*typed.routing_mut_for_test()) = Arc::new(routing);
    (*(*sealed.state_mut_for_test()).typed_mut_for_test()) = Arc::new(typed);
    moved_facts(sealed, false)
}

#[test]
fn a_collision_met_routing_again_makes_the_run_in_full() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let installed = WorthQueryInstalledPartitionedComputation::<_, _, Computation, _>::new(
        Rerouted::default(),
        ComputationRetention::ProducerOperation,
    );
    let fresh = || Some(ComputationPrior::new_for_test(edition(), Err(Cause::FirstRun), None));
    *(*installed.owner_for_test()).entries.lock().unwrap() = BTreeMap::from([(1, (0, 10)), (2, (1, 20))]);
    let first = attempt(&world, &installed, fresh())
        .sealed
        .unwrap()
        .unwrap();
    // Item 1 moves to key 2, where the prior holds item 2 under a foreign
    // digest; item 2 keeps its route.
    *(*installed.owner_for_test()).entries.lock().unwrap() = BTreeMap::from([(1, (2, 10)), (2, (1, 20))]);
    let prior = colliding(first, PartitionItemId(2), &Parity(2));
    let next = attempt(&world, &installed, Some(prior));
    let full = attempt(&world, &installed, fresh());
    assert!(full.outcome.is_ok(), "a fresh build meets no collision");
    assert_eq!(next.outcome, full.outcome, "the run made again in full");
    assert!(matches!(
        next.runs.as_slice(),
        [(Run::Full(Cause::IdentityCollision), Some(_))]
    ));
    assert_eq!(next.gathered, full.gathered);
    assert!(
        next.work.field_reads() > full.work.field_reads(),
        "the aborted routing reads remain observed before the complete retry"
    );
}

/// A digest-boundary adversary models the impossible-to-search-for SHA prefix
/// collision directly in retained routing. The whole sole-member partition,
/// including its key-dependent leaf, belongs to A; its compact identity is B's.
#[test]
fn replacing_a_sole_member_key_with_a_prefix_collision_matches_a_fresh_run() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let installed = WorthQueryInstalledPartitionedComputation::<_, _, Computation, _>::new(
        Rerouted::default(),
        ComputationRetention::ProducerOperation,
    );
    *(*installed.owner_for_test()).key_only.lock().unwrap() = true;
    *(*installed.owner_for_test()).entries.lock().unwrap() = BTreeMap::from([(1, (0, 10))]);
    let fresh = || Some(ComputationPrior::new_for_test(edition(), Err(Cause::FirstRun), None));
    let mut sealed = attempt(&world, &installed, fresh())
        .sealed
        .unwrap()
        .unwrap();
    let typed = std::mem::replace(&mut (*(*sealed.state_mut_for_test()).typed_mut_for_test()), Arc::new(()))
        .downcast::<RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let mut typed = Arc::try_unwrap(typed).ok().unwrap();
    let identity = (*typed.routing_for_test()).partition_of_for_test(PartitionItemId(1)).unwrap();
    let b_digest = digest_of(&Parity(0));
    let mut a_digest = b_digest;
    a_digest[31] ^= 1;
    assert_eq!(&a_digest[..8], &b_digest[..8]);
    assert_ne!(a_digest, b_digest);
    let partition = (*typed.partitions_for_test()).get_for_test(&identity).unwrap();
    let replacement = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::RetainedPartition::new_for_test(Arc::new(Parity(42)), (*partition.gather_for_test()).clone());
    let request = live_scope();
    let execution = QueryRequestExecution::open_for_test(
        RuntimeWorldExecutionPlacement::Serial(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy(
                std::num::NonZeroUsize::MIN, None,
            ),
        ),
        &request,
    );
    (*typed
        .partitions_mut_for_test())
        .put_for_test(
            identity,
            replacement,
            execution.allocation_policy_for_test().unwrap(),
        )
        .unwrap();
    execution
        .with_native_lease_for_test(|lease| {
            (*typed
                .tree_mut_for_test())
                .update_native(identity, 62, Some(lease), || execution.checkpoint_for_test())
        })
        .unwrap()
        .unwrap();
    let mut routing = ComputationPartitionRouting::default();
    routing
        .route_for_test(PartitionItemId(1), a_digest, |_| Ok(()))
        .unwrap();
    (*typed.routing_mut_for_test()) = Arc::new(routing);
    (*(*sealed.state_mut_for_test()).typed_mut_for_test()) = Arc::new(typed);
    // Only the key call read this fact. Membership, item value and gathering
    // stayed unchanged, so a compact-identity-only test would carry A's leaf.
    let (key, fact) = (*(*sealed
        .state_for_test())
        .facts_for_test())
        .facts_for_test()
        .find(|(_, _, readers)| {
            readers
                .reads()
                .any(|read| matches!(read, ComputationRead::ItemKey(_)))
        })
        .map(|(key, fact, _)| (key.clone(), fact.clone()))
        .unwrap();
    let WorthQueryApplicationObservedFact::Field { entity_id, .. } = fact else {
        panic!("key field");
    };
    (*(*sealed.state_mut_for_test()).facts_mut_for_test()).replace_fact_for_test(
        &key,
        WorthQueryApplicationObservedFact::SourceEntity { entity_id },
    );
    let prior = ComputationPrior::new_for_test(edition(), Ok(std::sync::Arc::new(sealed.state_into_for_test())), None);
    let next = attempt(&world, &installed, Some(prior));
    let full = attempt(&world, &installed, fresh());
    assert_eq!(next.outcome, full.outcome, "B must not carry A's result");
    assert_eq!(next.gathered, full.gathered, "gather receives B");
    let next = next.sealed.unwrap().unwrap();
    let full = full.sealed.unwrap().unwrap();
    let keys = |sealed: &SealedComputationRun| {
        (*(*(*sealed
            .state_for_test())
            .typed_for_test())
            .downcast_ref::<RetainedPartitions<Parity, Number, u64>>()
            .unwrap()
            .partitions_for_test())
            .values_for_test()
            .map(|partition| (*partition.key_for_test()).0)
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&next), keys(&full), "retained typed keys agree");
}
