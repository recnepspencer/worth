use super::{
    directory::RunDirectory, run::Run, AdmittedFactKey, RetainedFact, Slot, StoreDenial,
    AUTHOR_CHUNK,
};
use worth_execution::{ExecutionArray, ExecutionArrayBuilder};

pub(in crate::domain_computation::primary_graph) struct RetainedFactStore<T> {
    seed: Option<Run<T>>,
    chunk: Option<Run<T>>,
    directory: Option<RunDirectory<T>>,
    count: usize,
    failure: Option<StoreDenial>,
}
impl<T> RetainedFactStore<T> {
    pub(in crate::domain_computation::primary_graph) fn new(
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        policy.check_live()?;
        Ok(Self::vacant())
    }
    pub(in crate::domain_computation::primary_graph) fn vacant() -> Self {
        Self {
            seed: None,
            chunk: None,
            directory: None,
            count: 0,
            failure: None,
        }
    }
    /// Re-author a sealed canonical payload only when new facts must merge.
    /// The old owning iterator and its charge coexist with the admitted slots.
    pub(in crate::domain_computation::primary_graph) fn from_sorted_records(
        records: ExecutionArray<RetainedFact<T>>,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        let mut store = Self::new(policy)?;
        let count = records.len();
        let mut seed = Run::empty(count, policy)?;
        let mut records = records.into_iter();
        for index in 0..count {
            policy.check_live()?;
            *seed.slots[index].borrow_mut() = Some(records.next().unwrap());
            seed.used += 1;
        }
        drop(records);
        store.seed = Some(seed);
        store.count = count;
        Ok(store)
    }
    fn directory_slots(&self) -> &[std::cell::RefCell<Option<Run<T>>>] {
        self.directory
            .as_ref()
            .map(|directory| directory.slots.as_ref().unwrap().elements())
            .unwrap_or(&[])
    }

    pub(in crate::domain_computation::primary_graph) fn insert(
        &mut self,
        key: AdmittedFactKey,
        value: T,
        policy: super::StorageControl<'_, '_>,
        duplicate: impl FnOnce(&mut T, T) -> Result<(), StoreDenial>,
    ) -> Result<(), StoreDenial> {
        if let Some(denial) = &self.failure {
            return Err(denial.clone());
        }
        let result = self.insert_inner(key, value, policy, duplicate);
        if let Err(denial) = &result {
            self.failure = Some(denial.clone());
        }
        result
    }
    fn insert_inner(
        &mut self,
        key: AdmittedFactKey,
        value: T,
        policy: super::StorageControl<'_, '_>,
        duplicate: impl FnOnce(&mut T, T) -> Result<(), StoreDenial>,
    ) -> Result<(), StoreDenial> {
        policy.check_live()?;
        let Some(value) = self.resolve_existing(&key, value, duplicate, policy)? else {
            policy.check_live()?;
            return Ok(());
        };
        let count = self
            .count
            .checked_add(1)
            .ok_or(StoreDenial::Representability)?;
        if self.chunk.is_none() {
            self.chunk = Some(Run::empty(AUTHOR_CHUNK, policy)?);
        }
        let chunk = self.chunk.as_mut().unwrap();
        *chunk.slots[chunk.used].borrow_mut() = Some(RetainedFact {
            key,
            value,
            #[cfg(test)]
            ordinal: self.count,
        });
        chunk.used += 1;
        self.count = count;
        policy.check_live()?;
        if chunk.used == AUTHOR_CHUNK {
            self.flush(policy)?;
        }

        Ok(())
    }
    fn resolve_existing(
        &self,
        key: &AdmittedFactKey,
        value: T,
        duplicate: impl FnOnce(&mut T, T) -> Result<(), StoreDenial>,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Option<T>, StoreDenial> {
        let mut value = Some(value);
        let mut duplicate = Some(duplicate);
        for slot in self
            .chunk
            .iter()
            .flat_map(|chunk| chunk.slots.iter().take(chunk.used))
        {
            if slot
                .borrow()
                .as_ref()
                .unwrap()
                .key
                .compare(key, policy)?
                .is_eq()
            {
                Self::resolve_slot(slot, value.take().unwrap(), duplicate.take().unwrap())?;
                return Ok(None);
            }
        }
        if let Some(slot) = self
            .seed
            .as_ref()
            .map(|run| run.locate(key, policy))
            .transpose()?
            .flatten()
        {
            Self::resolve_slot(slot, value.take().unwrap(), duplicate.take().unwrap())?;
            return Ok(None);
        }
        for descriptor in self.directory_slots().iter() {
            policy.check_live()?;
            let descriptor = descriptor.borrow();
            if let Some(slot) = descriptor
                .as_ref()
                .map(|run| run.locate(key, policy))
                .transpose()?
                .flatten()
            {
                // Resolve inside the descriptor borrow; no reference escapes it.
                Self::resolve_slot(slot, value.take().unwrap(), duplicate.take().unwrap())?;
                return Ok(None);
            }
        }
        Ok(value)
    }
    fn resolve_slot(
        slot: &Slot<T>,
        value: T,
        duplicate: impl FnOnce(&mut T, T) -> Result<(), StoreDenial>,
    ) -> Result<(), StoreDenial> {
        let mut record = slot.borrow_mut();
        let record = record.as_mut().unwrap();
        duplicate(&mut record.value, value)
    }
    fn flush(&mut self, policy: super::StorageControl<'_, '_>) -> Result<(), StoreDenial> {
        let next = Run::empty(AUTHOR_CHUNK, policy)?;
        let mut carry = self.chunk.replace(next).unwrap();
        carry.sort_chunk(policy)?;
        let mut level = 0;
        loop {
            if self.directory.is_none() {
                self.directory = Some(RunDirectory::new(policy)?);
            }
            self.directory.as_mut().unwrap().ensure(level, policy)?;
            let slot = &self.directory_slots()[level];
            let previous = slot.borrow_mut().take();
            match previous {
                None => {
                    *slot.borrow_mut() = Some(carry);
                    return Ok(());
                }
                Some(previous) => carry = Run::merge(previous, carry, policy)?,
            }
            level = level.checked_add(1).ok_or(StoreDenial::Representability)?;
        }
    }
    pub(in crate::domain_computation::primary_graph) fn finish(
        self,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<ExecutionArray<RetainedFact<T>>, StoreDenial> {
        if let Some(denial) = &self.failure {
            return Err(denial.clone());
        }
        if let Some(chunk) = &self.chunk {
            chunk.sort_chunk(policy)?;
        }
        policy.check_live()?;
        let mut output = ExecutionArrayBuilder::allocate(self.count, policy.policy())?;
        for _ in 0..self.count {
            policy.check_live()?;
            let mut selected = None;
            for candidate in self
                .seed
                .iter()
                .map(|_| Head::Seed)
                .chain(self.chunk.iter().map(|_| Head::Chunk))
                .chain((0..self.directory_slots().len()).map(Head::Directory))
            {
                let present = match candidate {
                    Head::Directory(index) => self.directory_slots()[index]
                        .borrow()
                        .as_ref()
                        .and_then(Run::head)
                        .is_some(),
                    _ => self.with_run(candidate, |run| run.head().is_some()),
                };
                if present
                    && selected
                        .map(|previous| self.precedes(candidate, previous, policy))
                        .transpose()?
                        .unwrap_or(true)
                {
                    selected = Some(candidate);
                }
            }
            let record = self.with_run(selected.expect("exact retained count"), Run::take_head);
            output.push(record)?;
        }
        Ok(output.seal()?)
    }
    fn with_run<R>(&self, head: Head, read: impl FnOnce(&Run<T>) -> R) -> R {
        match head {
            Head::Seed => read(self.seed.as_ref().unwrap()),
            Head::Chunk => read(self.chunk.as_ref().unwrap()),
            Head::Directory(index) => {
                let descriptor = self.directory_slots()[index].borrow();
                match descriptor.as_ref() {
                    Some(run) => read(run),
                    None => unreachable!("empty descriptors skipped before with_run"),
                }
            }
        }
    }
    fn precedes(
        &self,
        first: Head,
        second: Head,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<bool, StoreDenial> {
        self.with_run(first, |first| {
            self.with_run(second, |second| {
                first
                    .head()
                    .unwrap()
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .key
                    .compare(
                        &second.head().unwrap().borrow().as_ref().unwrap().key,
                        policy,
                    )
                    .map(|order| order.is_lt())
            })
        })
    }
}
#[derive(Clone, Copy)]
enum Head {
    Seed,
    Chunk,
    Directory(usize),
}
