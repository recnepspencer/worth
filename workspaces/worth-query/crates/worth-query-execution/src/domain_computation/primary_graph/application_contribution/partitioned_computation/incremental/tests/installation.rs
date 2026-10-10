//! A run reuses only the state its own installation retained: another owner
//! over the same items, keys and results runs in full, and so does the same
//! owner installed again, whose code may have changed.

use super::*;

/// The test owner's computation with every result doubled: the same
/// associated types, another installed owner.
struct Doubling(Owner);

impl WorthQueryPartitionedComputationOwner<Schema, Feature, Computation> for Doubling {
    type Operation = TouchAccountOperation;
    type Item = Number;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;

    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
    ) -> Result<WorthQueryComputationPartitionPlan<Number>, WorthQueryComputationInputDenial<u32>>
    {
        self.0.partitions(reader, account)
    }

    fn partition_key(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        item: &Number,
    ) -> Result<Parity, WorthQueryComputationInputDenial<u32>> {
        self.0.partition_key(reader, account, item)
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        partition: WorthQueryComputationPartitionMembers<'_, Parity, Number>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        self.0.gather(reader, account, partition)
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Parity, u64>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        Ok(self.0.compute_partition(partition, checkpoint)? * 2)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        self.0.reducer()
    }

    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}

impl TestOwner for Doubling {
    fn gathered(&self) -> &Mutex<Vec<u64>> {
        &self.0.gathered
    }
}

#[test]
fn another_owner_with_the_same_types_runs_in_full() {
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let doubling = WorthQueryInstalledPartitionedComputation::new(
        Doubling(Owner {
            modulus: 2,
            status: StatusRead::Gather(1),
            reducer: sum,
            bump: Mutex::new(0),
            work: Mutex::new([1, 1]),
            gathered: Mutex::default(),
            calls: Mutex::default(),
        }),
        ComputationRetention::ProducerOperation,
    );

    let next = attempt(&world, &doubling, Some(prior_of(first, false)));
    assert!(matches!(
        next.runs.as_slice(),
        [(Run::Full(Cause::OtherInstallation), Some(_))]
    ));
    assert_eq!(next.gathered, [0, 1], "every partition is gathered");
    assert_eq!(
        next.outcome.unwrap().0,
        (1..=4).sum::<u64>() * 2,
        "its own total"
    );
    assert_eq!(*doubling.owner.0.calls.lock().unwrap(), full_call_law());
}

#[test]
fn a_reinstalled_owner_runs_in_full() {
    let world = installed_authorization_world(true);
    let first = first_run(&world, &installed(StatusRead::Gather(1), sum));
    let reinstalled = installed(StatusRead::Gather(1), sum);

    let next = attempt(&world, &reinstalled, Some(prior_of(first, false)));
    assert!(matches!(
        next.runs.as_slice(),
        [(Run::Full(Cause::OtherInstallation), Some(_))]
    ));
    assert_eq!(next.gathered, [0, 1], "every partition is gathered");
    assert_eq!(next.outcome.unwrap().0, (1..=4).sum::<u64>());
    assert_eq!(*reinstalled.owner.calls.lock().unwrap(), full_call_law());
}

#[test]
fn a_clone_of_the_installation_reuses_its_state() {
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);

    *installed.owner.calls.lock().unwrap() = [0; 4];
    let next = attempt(&world, &installed.clone(), Some(prior_of(first, false)));
    assert!(matches!(next.runs.as_slice(), [(Run::Incremental, None)]));
    assert!(next.gathered.is_empty(), "nothing is gathered again");
    assert_eq!(*installed.owner.calls.lock().unwrap(), [0; 4]);
}

#[test]
fn a_reinstalled_reducer_runs_in_full_with_its_own_reduction() {
    let world = installed_authorization_world(true);
    let first = first_run(&world, &installed(StatusRead::Gather(1), sum));
    let expected = (0..2)
        .map(|parity| (1..=4).filter(|item| item % 2 == parity).sum::<u64>())
        .max()
        .unwrap();
    let reinstalled = installed(StatusRead::Gather(1), |left, right| (*left).max(*right));
    let next = attempt(&world, &reinstalled, Some(prior_of(first, false)));
    assert!(matches!(
        next.runs.as_slice(),
        [(Run::Full(Cause::OtherInstallation), Some(_))]
    ));
    assert_eq!(next.gathered, [0, 1]);
    assert_eq!(next.outcome.unwrap().0, expected);
    assert_eq!(*reinstalled.owner.calls.lock().unwrap(), full_call_law());
}

/// A new installation meets each item and computes every distinct model key.
fn full_call_law() -> [usize; 4] {
    let items = (1..=4).collect::<Vec<_>>();
    let partitions = items
        .iter()
        .map(|item| item % 2)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    [1, items.len(), partitions, partitions]
}
