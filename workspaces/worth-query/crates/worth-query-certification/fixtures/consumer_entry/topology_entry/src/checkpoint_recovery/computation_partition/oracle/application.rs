//! The neutral partitioned application and independent owner-entry counters.
use super::*;

pub(super) struct OracleTotals<const WORK: usize = TOTALS_WORK>;
impl<const WORK: usize> ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>
    for OracleTotals<WORK>
{
    type Input = RegionEntries;
    type Output = RegionArtifact;
    type Partition = RegionKey;
    type Reuse = NoWarmStart;
    type Stopped = RegionStopped;
    const IDENTITY: &'static str = "checkpoint-region-output-totals";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(WORK, TOTALS_RETAINED_BYTES);
}

thread_local! { static KERNEL_CHARGES: std::cell::RefCell<Vec<u64>> = const { std::cell::RefCell::new(Vec::new()) }; }
pub(super) fn kernel_charges() -> Vec<u64> {
    KERNEL_CHARGES.with(|charges| std::mem::take(&mut *charges.borrow_mut()))
}

pub(super) static COMBINES: AtomicUsize = AtomicUsize::new(0);

pub(super) static PLANS: AtomicUsize = AtomicUsize::new(0);
pub(super) static KEYS: AtomicUsize = AtomicUsize::new(0);
pub(super) static GATHERS: AtomicUsize = AtomicUsize::new(0);
pub(super) static KERNELS: AtomicUsize = AtomicUsize::new(0);

/// How often a decision's run entered each of the owner's calls.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct OwnerCalls {
    pub(super) plans: usize,
    pub(super) keys: usize,
    pub(super) gathers: usize,
    pub(super) kernels: usize,
}

pub(super) fn take_calls() -> OwnerCalls {
    OwnerCalls {
        plans: PLANS.swap(0, Ordering::Relaxed),
        keys: KEYS.swap(0, Ordering::Relaxed),
        gathers: GATHERS.swap(0, Ordering::Relaxed),
        kernels: KERNELS.swap(0, Ordering::Relaxed),
    }
}

/// Totals each region in entry order, then the regions. An even region also
/// sums the weight its set lends it, so the weight is a fact every even
/// region gathers.
pub(super) struct OracleOwner<const MODE: u8 = 0>;
impl<const WORK: usize, const MODE: u8>
    WorthQueryPartitionedComputationOwner<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        OracleTotals<WORK>,
    > for OracleOwner<MODE>
{
    type Operation = TotalRegionOutput;
    type Item = Entry;
    type Gathered = Vec<EntryData>;
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = u32;

    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        set: &Set,
    ) -> Result<WorthQueryComputationPartitionPlan<Entry>, InputDenial> {
        PLANS.fetch_add(1, Ordering::Relaxed);
        Ok(facts::entries(reader, set)?)
    }

    fn partition_key(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        _: &Set,
        entry: &Entry,
    ) -> Result<RegionKey, InputDenial> {
        KEYS.fetch_add(1, Ordering::Relaxed);
        let region = facts::region(reader, entry)?;
        let memberships = if MODE == 1 {
            u32::try_from(facts::membership_count(reader, entry)?).unwrap()
        } else {
            0
        };
        Ok(RegionKey(region + memberships))
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        set: &Set,
        partition: WorthQueryComputationPartitionMembers<'_, RegionKey, Entry>,
    ) -> Result<Vec<EntryData>, InputDenial> {
        GATHERS.fetch_add(1, Ordering::Relaxed);
        let mut gathered = Vec::new();
        if partition.key().0.is_multiple_of(2) {
            gathered.push(EntryData {
                value: facts::weight(reader, set)?,
                work: 1,
                fault: None,
            });
        }
        gathered.extend(facts::gathered(reader, partition.items())?);
        Ok(gathered)
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, RegionKey, Vec<EntryData>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        KERNELS.fetch_add(1, Ordering::Relaxed);
        let outcome = owner::total_region(partition.key().0, partition.gathered(), checkpoint);
        if outcome.is_ok() {
            // A completed region passed every declared entry checkpoint.
            let work = partition
                .gathered()
                .iter()
                .map(|entry| entry.work as u64)
                .sum();
            KERNEL_CHARGES.with(|charges| charges.borrow_mut().push(work));
        }
        outcome
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(
            || -0.0,
            if MODE == 4 {
                tree_count::faulting_sum
            } else {
                tree_count::sum
            },
        )
    }

    fn complete(&self, reduced: f64) -> Result<f64, u32> {
        Ok(reduced)
    }
}
