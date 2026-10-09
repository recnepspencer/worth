use super::ordered::{OrderedStore, Run};
use crate::mvcc::RelationalTransactionStagingDenial as Denial;
use std::{cell::RefCell, sync::Arc};
use worth_execution::{ExecutionAllocationPolicy, ExecutionArray, ExecutionArrayBuilder};

const CHUNK: usize = 32; // algorithm granularity, never a logical quota
const LEVELS: usize = usize::BITS as usize; // binary counter width, never admission capacity

type Slots<T> = ExecutionArray<RefCell<Option<T>>>;

pub(crate) struct Author<'scope, 'authority, T> {
    original: OrderedStore<T>,
    levels: [Option<Run<T>>; LEVELS], // transient stack handles; no heap directory
    chunk: Option<Slots<T>>,
    used: usize,
    units: usize,
    len: usize,
    changed: bool,
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
}
impl<'scope, 'authority, T: Ord> Author<'scope, 'authority, T> {
    pub(crate) fn new(
        original: &OrderedStore<T>,
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
    ) -> Result<Self, Denial> {
        policy.check_live()?;
        let mut levels = std::array::from_fn(|_| None);
        for (index, run) in original.runs().iter().enumerate() {
            levels[index] = run.clone();
        }
        Ok(Self {
            original: original.clone(),
            levels,
            chunk: None,
            used: 0,
            units: original.units,
            len: original.len,
            changed: false,
            policy,
        })
    }
    pub(crate) fn insert(&mut self, value: T) -> Result<(), Denial> {
        self.policy.check_live()?;
        if self
            .levels
            .iter()
            .flatten()
            .any(|run| run.binary_search_by(|row| row.value().cmp(&value)).is_ok())
        {
            return Ok(());
        }
        if self.chunk.is_none() {
            let mut slots = ExecutionArrayBuilder::allocate(CHUNK, self.policy)?;
            for _ in 0..CHUNK {
                slots.push(RefCell::new(None))?;
            }
            self.chunk = Some(slots.seal()?);
        }
        let chunk = self.chunk.as_ref().expect("author chunk");
        let mut index = 0;
        while index < self.used {
            self.policy.check_live()?;
            let order = chunk[index]
                .borrow()
                .as_ref()
                .expect("initialized row")
                .cmp(&value);
            match order {
                std::cmp::Ordering::Equal => return Ok(()),
                std::cmp::Ordering::Greater => break,
                std::cmp::Ordering::Less => index += 1,
            }
        }
        let new_len = self.len.checked_add(1).ok_or(Denial::CardinalityOverflow)?;
        for position in (index..self.used).rev() {
            *chunk[position + 1].borrow_mut() = chunk[position].borrow_mut().take();
        }
        // Move actual typed payload into admitted slot backing. Nested input heaps remain excluded.
        *chunk[index].borrow_mut() = Some(value);
        self.used += 1;
        self.len = new_len;
        self.changed = true;
        self.policy.check_live()?;
        if self.used == CHUNK {
            self.flush()?;
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Denial> {
        if self.used == 0 {
            return Ok(());
        }
        let mut rows = ExecutionArrayBuilder::allocate(self.used, self.policy)?;
        let chunk = self.chunk.as_ref().expect("pending rows");
        for row in chunk.iter().take(self.used) {
            rows.push(row.borrow_mut().take().expect("initialized pending row"))?;
        }
        let values = Arc::new(rows.seal()?);
        let mut handles = ExecutionArrayBuilder::allocate(self.used, self.policy)?;
        for index in 0..self.used {
            handles.push(super::row::StoredRow::new(Arc::clone(&values), index))?;
        }
        let mut carry = Arc::new(handles.seal()?);
        self.chunk = None; // pending values/backing are gone before their reservation
        self.used = 0;
        let mut level = 0;
        let mut bits = self.units;
        let new_units = self
            .units
            .checked_add(1)
            .ok_or(Denial::CardinalityOverflow)?;
        while bits & 1 == 1 {
            self.policy.check_live()?;
            carry = merge(
                self.levels[level].as_ref().expect("occupied binary level"),
                &carry,
                self.policy,
            )?;
            self.levels[level] = None;
            bits >>= 1;
            level += 1;
        }
        self.levels[level] = Some(carry);
        self.units = new_units;
        Ok(())
    }
    pub(crate) fn finish(mut self) -> Result<OrderedStore<T>, Denial> {
        self.policy.check_live()?;
        if !self.changed {
            return Ok(self.original);
        }
        self.flush()?;
        let height = self
            .levels
            .iter()
            .rposition(Option::is_some)
            .map_or(0, |index| index + 1);
        let mut directory = ExecutionArrayBuilder::allocate(height, self.policy)?;
        for run in self.levels.iter_mut().take(height) {
            directory.push(run.take())?;
        }
        let directory = directory.seal()?;
        Ok(OrderedStore {
            directory: Some(Arc::new(directory)),
            units: self.units,
            len: self.len,
        })
    }
}
fn merge<T: Ord>(
    left: &Run<T>,
    right: &Run<T>,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<Run<T>, Denial> {
    let count = left
        .len()
        .checked_add(right.len())
        .ok_or(Denial::CardinalityOverflow)?;
    let mut rows = ExecutionArrayBuilder::allocate(count, policy)?;
    let (mut a, mut b) = (0, 0);
    while a < left.len() || b < right.len() {
        policy.check_live()?;
        if b == right.len() || (a < left.len() && left[a] <= right[b]) {
            rows.push(left[a].clone())?;
            a += 1;
        } else {
            rows.push(right[b].clone())?;
            b += 1;
        }
    }
    Ok(Arc::new(rows.seal()?))
}
