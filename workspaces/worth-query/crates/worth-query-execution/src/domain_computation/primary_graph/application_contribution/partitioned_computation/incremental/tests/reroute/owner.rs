//! The item-edit differential owner, with independent key and gather reads.
use super::super::*;
use super::item;
use std::collections::BTreeMap;

/// The items the membership reads: identity to key and value. The membership
/// reads the account's status, which the test moves every step; every key and
/// every gather reads the label, which it moves on some steps, so a removed
/// item or partition that lingered among its readers would show.
#[derive(Default)]
pub(in super::super) struct Rerouted {
    pub(in super::super) entries: Mutex<BTreeMap<u64, (u64, u64)>>,
    /// The identity of each item a key call keyed, in call order.
    pub(in super::super) keyed: Mutex<Vec<u64>>,
    gathered: Mutex<Vec<u64>>,
    pub(in super::super) key_only: Mutex<bool>,
}

impl WorthQueryPartitionedComputationOwner<Schema, Feature, Computation> for Rerouted {
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
        reader.field(account, AccountStatus::reference())?;
        let entries = self.entries.lock().unwrap().clone();
        Ok(WorthQueryComputationPartitionPlan::keyed(
            entries.into_iter().rev().map(|(id, entry)| item(id, entry)),
            |item| PartitionItemId(item.0 / 1_000_000),
        ))
    }

    fn partition_key(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        item: &Number,
    ) -> Result<Parity, WorthQueryComputationInputDenial<u32>> {
        reader.field(account, AccountLabel::reference())?;
        self.keyed.lock().unwrap().push(item.0 / 1_000_000);
        Ok(Parity(item.0 / 1_000 % 1_000))
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        partition: WorthQueryComputationPartitionMembers<'_, Parity, Number>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        if !*self.key_only.lock().unwrap() {
            reader.field(account, AccountLabel::reference())?;
        }
        self.gathered.lock().unwrap().push(partition.key().0);
        let key_value = if *self.key_only.lock().unwrap() {
            partition.key().0
        } else {
            0
        };
        Ok(key_value
            + partition
                .items()
                .map(|(identity, item)| (identity.0 + 1) * (item.0 % 1_000))
                .sum::<u64>())
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Parity, u64>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        let gathered = *partition.gathered();
        checkpoint.advance(usize::try_from(1 + gathered % 3).unwrap())?;
        Ok(gathered)
    }

    /// A combine that is neither associative nor commutative, so the root
    /// names the tree's shape as well as its leaves.
    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(
            || 1,
            |left, right| {
                left.wrapping_mul(1_000_003)
                    .wrapping_add(right.wrapping_mul(7))
                    .wrapping_add(1)
            },
        )
    }

    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}

impl TestOwner for Rerouted {
    fn gathered(&self) -> &Mutex<Vec<u64>> {
        &self.gathered
    }
}
