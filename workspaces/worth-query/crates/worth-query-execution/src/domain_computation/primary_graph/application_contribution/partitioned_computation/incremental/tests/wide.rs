//! Ten thousand partitions, where no topology budget applies: after one fact
//! a single partition read moves, the next run gathers and computes that
//! partition alone, carries every other owner call, and charges what a full
//! run charges.

use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Serialize;
use worth_query_declaration::facade::application_program::{
    ApplicationComputationExecution, ApplicationComputationPartition,
    ApplicationComputationResourceCeiling,
};

use super::super::super::attribution_tests::{Artifact, Reuse, Stopped};
use super::*;

const PARTITIONS: u64 = 10_000;

/// Every item is its own partition.
#[derive(Serialize)]
struct Slot(u64);
impl ApplicationComputationPartition for Slot {
    const IDENTITY: &'static str = "worth.query.tests.wide-slot.v1";
}

struct Wide;
impl ApplicationManagedComputation<Schema, Feature> for Wide {
    type Input = Input;
    type Output = Artifact;
    type Partition = Slot;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "worth.query.tests.wide-computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(1 << 32, 4096);
}

/// Counts every owner call. Only the gather of slot 1 reads a fact.
#[derive(Default)]
struct WideOwner {
    membership: AtomicUsize,
    keys: AtomicUsize,
    kernels: AtomicUsize,
    gathered: Mutex<Vec<u64>>,
}

impl WideOwner {
    /// The membership, item key and kernel calls since the last take.
    fn take_calls(&self) -> [usize; 3] {
        [&self.membership, &self.keys, &self.kernels].map(|calls| calls.swap(0, Ordering::SeqCst))
    }
}

impl WorthQueryPartitionedComputationOwner<Schema, Feature, Wide> for WideOwner {
    type Operation = TouchAccountOperation;
    type Item = u64;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;

    fn partitions(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Root,
    ) -> Result<WorthQueryComputationPartitionPlan<u64>, WorthQueryComputationInputDenial<u32>>
    {
        self.membership.fetch_add(1, Ordering::SeqCst);
        Ok(WorthQueryComputationPartitionPlan::keyed(
            1..=PARTITIONS,
            |item| PartitionItemId(*item),
        ))
    }

    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Root,
        item: &u64,
    ) -> Result<Slot, WorthQueryComputationInputDenial<u32>> {
        self.keys.fetch_add(1, Ordering::SeqCst);
        Ok(Slot(*item))
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        partition: WorthQueryComputationPartitionMembers<'_, Slot, u64>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        let slot = partition.key().0;
        if slot == 1 {
            reader.field(account, AccountStatus::reference())?;
        }
        self.gathered.lock().unwrap().push(slot);
        Ok(partition.items().map(|(_, item)| *item).sum())
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Slot, u64>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        self.kernels.fetch_add(1, Ordering::SeqCst);
        checkpoint.advance(1)?;
        Ok(*partition.gathered())
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, sum)
    }

    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}

impl TestOwner<Wide> for WideOwner {
    fn gathered(&self) -> &Mutex<Vec<u64>> {
        &self.gathered
    }
}

#[test]
fn ten_thousand_partitions_gather_and_compute_only_the_one_whose_fact_moved() {
    let world = installed_authorization_world(true);
    let installed = WorthQueryInstalledPartitionedComputation::<_, _, Wide, _>::new(
        WideOwner::default(),
        ComputationRetention::ProducerOperation,
    );
    let first = attempt(
        &world,
        &installed,
        Some(ComputationPrior::new(
            edition(),
            Err(Cause::NoPriorRecord),
            None,
        )),
    );
    assert!(matches!(
        first.runs.as_slice(),
        [(Run::Full(Cause::NoPriorRecord), Some(_))]
    ));
    let partitions = usize::try_from(PARTITIONS).unwrap();
    assert_eq!(first.gathered.len(), partitions);
    assert_eq!(installed.owner.take_calls(), [1, partitions, partitions]);
    let outcome = *first.outcome.as_ref().unwrap();
    assert_eq!(outcome.0, PARTITIONS * (PARTITIONS + 1) / 2);

    let mut state = first
        .sealed
        .unwrap()
        .expect("a producer's run retains")
        .state;
    let (key, entity_id) = state
        .facts
        .facts()
        .find_map(|(key, fact, _)| match fact {
            WorthQueryApplicationObservedFact::Field { entity_id, .. } => {
                Some((key.clone(), *entity_id))
            }
            _ => None,
        })
        .expect("slot 1 reads the status");
    state.facts.replace_fact(
        &key,
        WorthQueryApplicationObservedFact::SourceEntity { entity_id },
    );
    let prior = ComputationPrior::new(edition(), Ok(Arc::new(state)), None);

    let next = attempt(&world, &installed, Some(prior));
    assert!(matches!(next.runs.as_slice(), [(Run::Incremental, None)]));
    assert_eq!(next.gathered, [1]);
    assert_eq!(installed.owner.take_calls(), [0, 0, 1]);
    assert_eq!(
        next.outcome.unwrap(),
        outcome,
        "the same total, charged the same work"
    );
}
