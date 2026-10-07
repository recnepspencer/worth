//! An item routed again into a partition identity that another key digest
//! holds makes the run again in full. The retained routing is made to hold
//! a kept item under a digest that shares the moved item's partition identity
//! and is not its digest, which no real run retains: a fresh build of the
//! same items meets no collision, and the run made again equals it, charging
//! the reader what it charges.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_operation::application_computation_partition_identity;

use super::super::super::routing::ComputationPartitionRouting;
use super::super::retained::RetainedPartitions;
use super::reroute::{moved_facts, Rerouted};
use super::*;

/// The key digest of `key`.
fn digest_of(key: &Parity) -> [u8; 32] {
    let identity = application_computation_partition_identity(key, &mut |_| Ok::<(), ()>(()));
    *identity.expect("the key encodes").digest()
}

/// The prior a producer hands its next run from `sealed`, its routing
/// holding `kept` under a digest that shares `key`'s partition identity and
/// is not `key`'s digest. The items live outside the facts, so the
/// membership's fact is moved for the run to plan them again.
fn colliding(
    mut sealed: SealedComputationRun,
    kept: PartitionItemId,
    key: &Parity,
) -> ComputationPrior {
    let typed = std::mem::replace(&mut sealed.state.typed, Arc::new(()))
        .downcast::<RetainedPartitions<Parity, Number, u64>>()
        .expect("the run retains its partitions");
    let mut typed = Arc::try_unwrap(typed)
        .ok()
        .expect("the sealed run holds its partitions alone");
    let mut foreign = digest_of(key);
    foreign[31] ^= 1;
    let mut routing = ComputationPartitionRouting::default();
    for (identity, partition) in &typed.partitions {
        for item in typed.routing.members(*identity) {
            let digest = if item == kept {
                foreign
            } else {
                digest_of(&partition.key)
            };
            routing
                .route(item, digest, |_| Ok(()))
                .expect("the item routes");
        }
    }
    typed.routing = Arc::new(routing);
    sealed.state.typed = Arc::new(typed);
    moved_facts(sealed, false)
}

#[test]
fn a_collision_met_routing_again_makes_the_run_in_full() {
    let world = installed_authorization_world(true);
    let installed = WorthQueryInstalledPartitionedComputation::<_, _, Computation, _>::new(
        Rerouted::default(),
        ComputationRetention::ProducerOperation,
    );
    let fresh = || Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None));
    *installed.owner.entries.lock().unwrap() = BTreeMap::from([(1, (0, 10)), (2, (1, 20))]);
    let first = attempt(&world, &installed, fresh())
        .sealed
        .unwrap()
        .unwrap();
    // Item 1 moves to key 2, where the prior holds item 2 under a foreign
    // digest; item 2 keeps its route.
    *installed.owner.entries.lock().unwrap() = BTreeMap::from([(1, (2, 10)), (2, (1, 20))]);
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
    assert_eq!(next.work, full.work, "the reader charged a fresh build");
}

/// A digest-boundary adversary models the impossible-to-search-for SHA prefix
/// collision directly in retained routing. The whole sole-member partition,
/// including its key-dependent leaf, belongs to A; its compact identity is B's.
#[test]
fn replacing_a_sole_member_key_with_a_prefix_collision_matches_a_fresh_run() {
    let world = installed_authorization_world(true);
    let installed = WorthQueryInstalledPartitionedComputation::<_, _, Computation, _>::new(
        Rerouted::default(),
        ComputationRetention::ProducerOperation,
    );
    *installed.owner.key_only.lock().unwrap() = true;
    *installed.owner.entries.lock().unwrap() = BTreeMap::from([(1, (0, 10))]);
    let fresh = || Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None));
    let mut sealed = attempt(&world, &installed, fresh())
        .sealed
        .unwrap()
        .unwrap();
    let typed = std::mem::replace(&mut sealed.state.typed, Arc::new(()))
        .downcast::<RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let mut typed = Arc::try_unwrap(typed).ok().unwrap();
    let identity = typed.routing.partition_of(PartitionItemId(1)).unwrap();
    let b_digest = digest_of(&Parity(0));
    let mut a_digest = b_digest;
    a_digest[31] ^= 1;
    assert_eq!(&a_digest[..8], &b_digest[..8]);
    assert_ne!(a_digest, b_digest);
    let partition = Arc::get_mut(typed.partitions.get_mut(&identity).unwrap()).unwrap();
    partition.key = Arc::new(Parity(42));
    typed
        .tree
        .update_checked(identity, 62, u64::MAX, || Ok::<(), ()>(()))
        .unwrap();
    let mut routing = ComputationPartitionRouting::default();
    routing
        .route(PartitionItemId(1), a_digest, |_| Ok(()))
        .unwrap();
    typed.routing = Arc::new(routing);
    sealed.state.typed = Arc::new(typed);
    // Only the key call read this fact. Membership, item value and gathering
    // stayed unchanged, so a compact-identity-only test would carry A's leaf.
    let (key, fact) = sealed
        .state
        .facts
        .facts()
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
    sealed.state.facts.replace_fact(
        &key,
        WorthQueryApplicationObservedFact::SourceEntity { entity_id },
    );
    let prior = ComputationPrior::new(edition(), Ok(Arc::new(sealed.state)), None);
    let next = attempt(&world, &installed, Some(prior));
    let full = attempt(&world, &installed, fresh());
    assert_eq!(next.outcome, full.outcome, "B must not carry A's result");
    assert_eq!(next.gathered, full.gathered, "gather receives B");
    let next = next.sealed.unwrap().unwrap();
    let full = full.sealed.unwrap().unwrap();
    let keys = |sealed: &SealedComputationRun| {
        sealed
            .state
            .typed
            .downcast_ref::<RetainedPartitions<Parity, Number, u64>>()
            .unwrap()
            .partitions
            .values()
            .map(|partition| partition.key.0)
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&next), keys(&full), "retained typed keys agree");
}
