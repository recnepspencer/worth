//! Proposed retained SourceAdjacencyRevision nested owner. Input source storage
//! remains separately owned; every NEW capture/sort/union backing is admitted.
use super::StoreDenial;
use std::{cell::Cell, ops::Deref, sync::Arc};
use worth_execution::{ExecutionArray, ExecutionArrayBuilder};
use worth_relational::facade::identity::EntityId;

#[derive(Debug)]
pub struct AdmittedAdjacencyEndpoints {
    storage: Option<Arc<ExecutionArray<EntityId>>>,
    sorted_unique: bool,
}
impl Clone for AdmittedAdjacencyEndpoints {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.as_ref().map(Arc::clone),
            sorted_unique: self.sorted_unique,
        }
    }
}
impl Deref for AdmittedAdjacencyEndpoints {
    type Target = [EntityId];
    fn deref(&self) -> &[EntityId] {
        self.storage
            .as_ref()
            .map_or(&[], |storage| storage.elements())
    }
}
impl<'a> IntoIterator for &'a AdmittedAdjacencyEndpoints {
    type Item = &'a EntityId;
    type IntoIter = std::slice::Iter<'a, EntityId>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl AdmittedAdjacencyEndpoints {
    /// Admission of a new retained payload. Input buffers remain separate owners.
    pub fn from_observed(
        input: &[EntityId],
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
    ) -> Result<Self, crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    {
        Self::preserve(
            input,
            super::StorageControl::new(allocation_policy, request),
        )
        .map_err(|denial| denial.into_attempt_denial("source adjacency endpoints"))
    }

    /// No payload, allocation, ticket, hidden system policy, or allocator pool.
    pub fn empty() -> Self {
        Self {
            storage: None,
            sorted_unique: true,
        }
    }

    /// Preserve the accepted ordered endpoint body, including imported wire
    /// order. Every new retained backing is admitted before copying input.
    pub(in crate::domain_computation::primary_graph) fn preserve(
        input: &[EntityId],
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        policy.check_live()?;
        let mut output = ExecutionArrayBuilder::allocate(input.len(), policy.policy())?;
        let mut previous = None;
        let mut sorted_unique = true;
        for id in input {
            policy.check_live()?;
            if previous.is_some_and(|previous| previous >= *id) {
                sorted_unique = false;
            }
            output.push(*id)?;
            previous = Some(*id);
        }
        Ok(Self {
            storage: Some(Arc::new(output.seal()?)),
            sorted_unique,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn from_exact_iterator(
        count: usize,
        input: impl IntoIterator<Item = EntityId>,
        control: super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        control.check_live()?;
        let mut output = ExecutionArrayBuilder::allocate(count, control.policy())?;
        let mut previous = None;
        let mut sorted_unique = true;
        for id in input {
            control.check_live()?;
            if previous.is_some_and(|previous| previous >= id) {
                sorted_unique = false;
            }
            output.push(id)?;
            previous = Some(id);
        }
        Ok(Self {
            storage: Some(Arc::new(output.seal()?)),
            sorted_unique,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn capture(
        input: &[EntityId],
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        // Do not first clone the input Vec. Temporary sort and final exact unique
        // backing coexist with each other and with the borrowed input owner.
        policy.check_live()?;
        let mut sorted = ExecutionArrayBuilder::allocate(input.len(), policy.policy())?;
        for id in input {
            policy.check_live()?;
            sorted.push(Cell::new(*id))?;
        }
        let sorted = sorted.seal()?;
        heap_sort(&sorted, policy)?;
        let mut count = 0usize;
        let mut last = None;
        for id in sorted.iter().map(Cell::get) {
            policy.check_live()?;
            if last != Some(id) {
                count = count.checked_add(1).ok_or(StoreDenial::Representability)?;
                last = Some(id);
            }
        }
        policy.check_live()?;
        let mut output = ExecutionArrayBuilder::allocate(count, policy.policy())?;
        last = None;
        for id in sorted.iter().map(Cell::get) {
            policy.check_live()?;
            if last != Some(id) {
                output.push(id)?;
                last = Some(id);
            }
        }
        Ok(Self {
            storage: Some(Arc::new(output.seal()?)),
            sorted_unique: true,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn union(
        &self,
        other: &Self,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        let left_sorted = (!self.sorted_unique)
            .then(|| Self::capture(self, policy))
            .transpose()?;
        let right_sorted = (!other.sorted_unique)
            .then(|| Self::capture(other, policy))
            .transpose()?;
        let left = left_sorted.as_ref().unwrap_or(self);
        let right = right_sorted.as_ref().unwrap_or(other);
        let mut count = 0usize;
        union_visit(left, right, policy, |_| {
            count = count.checked_add(1).ok_or(StoreDenial::Representability)?;
            Ok(())
        })?;
        policy.check_live()?;
        let mut output = ExecutionArrayBuilder::allocate(count, policy.policy())?;
        union_visit(left, right, policy, |id| Ok(output.push(id)?))?;
        Ok(Self {
            storage: Some(Arc::new(output.seal()?)),
            sorted_unique: true,
        })
    }
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn charged_payload_bytes(
        &self,
    ) -> Option<u64> {
        self.storage
            .as_ref()
            .and_then(|storage| storage.charged_payload_bytes())
    }
}
fn union_visit(
    left: &[EntityId],
    right: &[EntityId],
    policy: super::StorageControl<'_, '_>,
    mut emit: impl FnMut(EntityId) -> Result<(), StoreDenial>,
) -> Result<(), StoreDenial> {
    let (mut a, mut b) = (0, 0);
    while a < left.len() || b < right.len() {
        policy.check_live()?;
        let id = match (left.get(a), right.get(b)) {
            (Some(x), Some(y)) if x == y => {
                a += 1;
                b += 1;
                *x
            }
            (Some(x), Some(y)) if x < y => {
                a += 1;
                *x
            }
            (Some(_), Some(y)) | (None, Some(y)) => {
                b += 1;
                *y
            }
            (Some(x), None) => {
                a += 1;
                *x
            }
            (None, None) => unreachable!("checked union cursors"),
        };
        emit(id)?;
    }
    policy.check_live()?;
    Ok(())
}
fn heap_sort(
    values: &[Cell<EntityId>],
    policy: super::StorageControl<'_, '_>,
) -> Result<(), StoreDenial> {
    for root in (0..values.len() / 2).rev() {
        sift(values, root, values.len(), policy)?;
    }
    for end in (1..values.len()).rev() {
        swap(values, 0, end);
        sift(values, 0, end, policy)?;
    }
    Ok(())
}
fn sift(
    values: &[Cell<EntityId>],
    mut root: usize,
    end: usize,
    policy: super::StorageControl<'_, '_>,
) -> Result<(), StoreDenial> {
    loop {
        policy.check_live()?;
        let Some(mut child) = root.checked_mul(2).and_then(|value| value.checked_add(1)) else {
            return Ok(());
        };
        if child >= end {
            return Ok(());
        }
        if child + 1 < end && values[child].get() < values[child + 1].get() {
            child += 1;
        }
        if values[root].get() >= values[child].get() {
            return Ok(());
        }
        swap(values, root, child);
        root = child;
    }
}
fn swap(values: &[Cell<EntityId>], left: usize, right: usize) {
    let id = values[left].get();
    values[left].set(values[right].get());
    values[right].set(id);
}

impl PartialEq for AdmittedAdjacencyEndpoints {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl Eq for AdmittedAdjacencyEndpoints {}
impl AsRef<[EntityId]> for AdmittedAdjacencyEndpoints {
    fn as_ref(&self) -> &[EntityId] {
        self.storage
            .as_ref()
            .map_or(&[], |storage| storage.elements())
    }
}
