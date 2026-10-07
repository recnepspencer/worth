//! Actual fixture owner: numbered items have declared weighted contributions.
use super::*;
use crate::checkpoint_recovery::parallel_history::reuse_cases::SHARED_WORK;
use worth_execution::PartitionItemId;
pub(super) struct HistoryOwner;
impl WorthQueryPartitionedComputationOwner<CheckpointSchema, PlanarFinalOutputFeature, OracleTotals>
    for HistoryOwner
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
        Ok(RegionKey(facts::region(reader, entry)?))
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
                work: SHARED_WORK as usize,
                fault: None,
            });
        }
        for (_, entry) in partition.items() {
            let mut data = facts::gathered(
                reader,
                std::iter::once((PartitionItemId(entry.number), entry)),
            )?
            .remove(0);
            data.value *= (entry.number + 1) as f64;
            data.work *= (entry.number + 1) as usize;
            gathered.push(data);
        }
        Ok(gathered)
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, RegionKey, Vec<EntryData>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        KERNELS.fetch_add(1, Ordering::Relaxed);
        owner::total_region(partition.key().0, partition.gathered(), checkpoint)
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(
            || -0.0,
            |left, right| {
                COMBINES.fetch_add(1, Ordering::Relaxed);
                left + right
            },
        )
    }
    fn complete(&self, reduced: f64) -> Result<f64, u32> {
        Ok(reduced)
    }
}
