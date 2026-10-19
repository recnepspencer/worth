//! One original fact backing carries codec admission and shared custody discovery.
//! Clones preserve the allocation; custody never certifies source currentness.
use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use std::ops::Deref;
use std::sync::{Mutex, Weak};
use worth_execution::{ExecutionAllocationPolicy, ExecutionMemoryReservation, MemoryLimitLevel};

#[derive(Debug)]
pub(in crate::domain_computation) struct CodecPayloadReservation {
    _funding: Option<ExecutionMemoryReservation>,
}
impl CodecPayloadReservation {
    pub(in crate::domain_computation::primary_graph) fn admit(
        control: StorageControl<'_, '_>,
        bytes: usize,
    ) -> Result<Arc<Self>, FactDecodeDenial> {
        control.check_live().map_err(FactDecodeDenial::Retention)?;
        let quote = u64::try_from(bytes).map_err(|_| FactDecodeDenial::Retention(
            crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Representability))?;
        let held = match control.policy() {
            ExecutionAllocationPolicy::SystemAllocation => None,
            ExecutionAllocationPolicy::Execution(lease) => Some(lease.reserve_memory(quote).map_err(|cause| {
                use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
                FactDecodeDenial::Execution(match cause.level {
                    MemoryLimitLevel::Policy { ancestor } => Native::PolicyMemoryExhausted { requested: cause.requested, admitted: cause.admitted, ancestor },
                    MemoryLimitLevel::Process => Native::ProcessMemoryExhausted { requested: cause.requested, admitted: cause.admitted },
                    MemoryLimitLevel::Declared => Native::DeclaredMemoryExhausted { requested: cause.requested, admitted: cause.admitted },
                })
            })?),
        };
        control.check_live().map_err(FactDecodeDenial::Retention)?;
        Ok(Arc::new(Self { _funding: held }))
    }
}
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct FactPayload {
    facts: Arc<[Fact]>,
    retention: FactRetentionStorage,
    _funding: Option<Arc<CodecPayloadReservation>>,
}

/// Fixed original payload metadata, allocated during native pre-effect preparation.
#[derive(Clone, Debug, Default)]
pub(in crate::domain_computation::primary_graph) struct FactRetentionStorage(
    Arc<Mutex<FactRetentionSlot>>,
);

/// Only external fact holders own the strong payload ticket. The payload keeps
/// weak discovery plus custody of the small cache allocation until it is gone.
#[derive(Debug, Default)]
pub(in crate::domain_computation::primary_graph) struct FactRetentionSlot {
    pub(in crate::domain_computation::primary_graph) allocation: Weak<RetainedInvalidationCapacity>,
    pub(in crate::domain_computation::primary_graph) storage:
        Option<Arc<RetainedInvalidationCapacity>>,
}

impl FactPayload {
    pub(super) fn decoded(facts: Vec<Fact>, funding: Arc<CodecPayloadReservation>) -> Self {
        Self {
            facts: facts.into(),
            _funding: Some(funding),
            retention: FactRetentionStorage::default(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn from_prepared(
        facts: Vec<Fact>,
        retention: FactRetentionStorage,
    ) -> Self {
        Self {
            facts: facts.into(),
            retention,
            _funding: None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn same_backing(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.facts, &other.facts)
    }
    pub(in crate::domain_computation::primary_graph) fn lock_retention(
        &self,
    ) -> std::sync::MutexGuard<'_, FactRetentionSlot> {
        self.retention
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// The cache, its own capacity ticket, and one expired weak payload-ticket
    /// block remain allocated even after the last external facts holder leaves.
    pub(in crate::domain_computation::primary_graph) fn retention_storage_bytes() -> Option<u64> {
        arc_bytes::<Mutex<FactRetentionSlot>>()?
            .checked_add(arc_bytes::<RetainedInvalidationCapacity>()?.checked_mul(2)?)
    }
}
// Native execution already owns its original fact admission; this entry does
// not accept decoded values or retrospectively fund an external allocation.
impl From<Arc<[Fact]>> for FactPayload {
    fn from(facts: Arc<[Fact]>) -> Self {
        Self {
            facts,
            _funding: None,
            retention: FactRetentionStorage::default(),
        }
    }
}
impl From<Vec<Fact>> for FactPayload {
    fn from(facts: Vec<Fact>) -> Self {
        Self::from(Arc::<[Fact]>::from(facts))
    }
}
impl Deref for FactPayload {
    type Target = [Fact];
    fn deref(&self) -> &[Fact] {
        &self.facts
    }
}
impl AsRef<[Fact]> for FactPayload {
    fn as_ref(&self) -> &[Fact] {
        self
    }
}
impl Eq for FactPayload {}
impl PartialEq for FactPayload {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

/// Wire10 contains scalar AspectValue only. Decoded strings together occupy at
/// most the source bytes; serde's growing unescape scratch is below twice that.
/// A locator field uses >=9 wire bytes and one exact FieldKey slot; indexed
/// definitions clone that locator once. An endpoint uses >=16 wire bytes and
/// one temporary EntityId slot (its retained native array has separate funding).
/// Include both Vec and Arc inline Fact payloads while their move overlaps.
pub(super) fn parse_quote(length: usize, count: usize) -> Option<usize> {
    length
        .checked_mul(3)?
        .checked_add((length / 9).checked_mul(2 * std::mem::size_of::<FieldKey>())?)?
        .checked_add((length / 16).checked_mul(std::mem::size_of::<EntityId>())?)?
        .checked_add(count.checked_mul(2 * std::mem::size_of::<Fact>())?)
        .and_then(|bytes| {
            bytes.checked_add(usize::try_from(FactPayload::retention_storage_bytes()?).ok()?)
        })
}

fn arc_bytes<T>() -> Option<u64> {
    let alignment = std::mem::align_of::<T>().max(std::mem::align_of::<usize>());
    let header = (2 * std::mem::size_of::<usize>()).div_ceil(alignment) * alignment;
    u64::try_from(
        header
            .checked_add(std::mem::size_of::<T>())?
            .div_ceil(alignment)
            .checked_mul(alignment)?,
    )
    .ok()
}
