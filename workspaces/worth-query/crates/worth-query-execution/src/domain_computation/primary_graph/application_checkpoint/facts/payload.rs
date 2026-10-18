//! Decoded native facts retain their deep parse admission through every clone.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use worth_execution::{ExecutionAllocationPolicy, ExecutionMemoryReservation, MemoryLimitLevel};
mod chunk;
mod concatenation;
mod iteration;
mod native;
mod patch;
mod sequence;
#[cfg(test)]
mod tests;
pub(in crate::domain_computation::primary_graph) use iteration::FactIter;
pub(in crate::domain_computation::primary_graph) use native::NativeFactPayloadBuilder;
pub(in crate::domain_computation::primary_graph) use sequence::FactSequence;

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
    storage: FactStorage,
    start: usize,
    length: usize,
    storage_bytes: Option<u64>,
    resident_bytes: Option<u64>,
    height: usize,
}
#[derive(Clone, Debug)]
enum FactStorage {
    Original(Arc<[Fact]>),
    Decoded {
        facts: Arc<[Fact]>,
        _funding: Arc<CodecPayloadReservation>,
    },
    Native(Arc<chunk::NativeFactChunk>),
    Joined {
        children: Arc<worth_execution::ExecutionArray<FactPayload>>,
    },
}
impl FactPayload {
    pub(super) fn decoded(facts: Vec<Fact>, funding: Arc<CodecPayloadReservation>) -> Self {
        let length = facts.len();
        let storage_bytes = original_storage_bytes(length).and_then(|bytes| {
            bytes.checked_add(
                crate::domain_computation::primary_graph::output_lineage::invalidation::arc_bytes::<
                    CodecPayloadReservation,
                >()?,
            )
        });
        let resident_bytes = resident_bytes(&facts, storage_bytes);
        Self {
            storage_bytes,
            resident_bytes,
            storage: FactStorage::Decoded {
                facts: facts.into(),
                _funding: funding,
            },
            start: 0,
            length,
            height: 0,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn same_backing(&self, other: &Self) -> bool {
        self.start == other.start
            && self.length == other.length
            && match (&self.storage, &other.storage) {
                (FactStorage::Original(a), FactStorage::Original(b)) => Arc::ptr_eq(a, b),
                (FactStorage::Decoded { facts: a, .. }, FactStorage::Decoded { facts: b, .. }) => {
                    Arc::ptr_eq(a, b)
                }
                (FactStorage::Native(a), FactStorage::Native(b)) => Arc::ptr_eq(a, b),
                (
                    FactStorage::Joined { children: a, .. },
                    FactStorage::Joined { children: b, .. },
                ) => Arc::ptr_eq(a, b),
                _ => false,
            }
    }
    pub(in crate::domain_computation::primary_graph) fn retained_storage_bytes(
        &self,
    ) -> Option<u64> {
        self.storage_bytes
    }
    /// Whole retained backing and nested payload, conservatively counting
    /// shared owners per held path; no global deduplication promise.
    pub(in crate::domain_computation::primary_graph) fn retained_resident_bytes(
        &self,
    ) -> Option<u64> {
        self.resident_bytes
    }
    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        self.length
    }
    pub(in crate::domain_computation::primary_graph) fn is_empty(&self) -> bool {
        self.length == 0
    }
    pub(in crate::domain_computation::primary_graph) fn get(
        &self,
        ordinal: usize,
    ) -> Option<&Fact> {
        if ordinal >= self.length {
            return None;
        }
        let (start, chunk) = self.chunk_at(ordinal);
        chunk.get(ordinal - start)
    }
    pub(in crate::domain_computation::primary_graph) fn iter(&self) -> FactIter<'_> {
        FactIter::new(self)
    }
    pub(in crate::domain_computation::primary_graph) fn checked_slice(
        &self,
        range: std::ops::Range<usize>,
    ) -> Option<Self> {
        (range.start <= range.end && range.end <= self.length).then(|| self.slice(range))
    }
    /// A view retains the complete original backing and all of its admission.
    /// Neither a range nor data equality establishes a new source binding.
    pub(in crate::domain_computation::primary_graph) fn slice(
        &self,
        range: std::ops::Range<usize>,
    ) -> Self {
        assert!(range.start <= range.end && range.end <= self.length);
        Self {
            storage_bytes: self.storage_bytes,
            resident_bytes: self.resident_bytes,
            storage: self.storage.clone(),
            start: self.start + range.start,
            length: range.len(),
            height: self.height,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn native(
        facts: worth_execution::ExecutionArray<Fact>,
    ) -> Self {
        Self::copied_native(facts, None, None)
    }

    fn joined(children: worth_execution::ExecutionArray<Self>) -> Self {
        assert_eq!(children.len(), 2);
        let length = children[0]
            .len()
            .checked_add(children[1].len())
            .expect("prepared fact count");
        let storage_bytes = array_storage_bytes::<Self>(2).and_then(|bytes| {
            bytes
                .checked_add(children[0].storage_bytes?)?
                .checked_add(children[1].storage_bytes?)
        });
        let resident_bytes = array_storage_bytes::<Self>(2).and_then(|bytes| {
            bytes
                .checked_add(children[0].resident_bytes?)?
                .checked_add(children[1].resident_bytes?)
        });
        let height = children[0].height.max(children[1].height) + 1;
        Self {
            storage_bytes,
            resident_bytes,
            storage: FactStorage::Joined {
                children: Arc::new(children),
            },
            start: 0,
            length,
            height,
        }
    }
    fn chunk_at(&self, ordinal: usize) -> (usize, &[Fact]) {
        let ordinal = self.start + ordinal;
        let (base, chunk) = match &self.storage {
            FactStorage::Original(facts) | FactStorage::Decoded { facts, .. } => {
                (0, facts.as_ref())
            }
            FactStorage::Native(chunk) => (0, chunk.facts.elements()),
            FactStorage::Joined { children, .. } => {
                let left = children[0].len();
                if ordinal < left {
                    children[0].chunk_at(ordinal)
                } else {
                    let (base, chunk) = children[1].chunk_at(ordinal - left);
                    (left + base, chunk)
                }
            }
        };
        let begin = base.max(self.start);
        let end = (base + chunk.len()).min(self.start + self.length);
        (begin - self.start, &chunk[begin - base..end - base])
    }
}
// Native execution already owns its original fact admission; this entry does
// not accept decoded values or retrospectively fund an external allocation.
impl From<Arc<[Fact]>> for FactPayload {
    fn from(facts: Arc<[Fact]>) -> Self {
        let length = facts.len();
        let storage_bytes = original_storage_bytes(length);
        let resident_bytes = resident_bytes(&facts, storage_bytes);
        Self {
            storage_bytes,
            resident_bytes,
            storage: FactStorage::Original(facts),
            start: 0,
            length,
            height: 0,
        }
    }
}
impl std::ops::Index<usize> for FactPayload {
    type Output = Fact;
    fn index(&self, ordinal: usize) -> &Fact {
        self.get(ordinal).expect("fact ordinal in its backing")
    }
}
impl<'a> IntoIterator for &'a FactPayload {
    type Item = &'a Fact;
    type IntoIter = FactIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl Eq for FactPayload {}
impl PartialEq for FactPayload {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
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
}

fn original_storage_bytes(count: usize) -> Option<u64> {
    use crate::domain_computation::primary_graph::output_lineage::invalidation::arc_bytes;
    arc_bytes::<[Fact; 0]>()?
        .checked_add(u64::try_from(std::mem::size_of::<Fact>().checked_mul(count)?).ok()?)
}
fn array_storage_bytes<T>(count: usize) -> Option<u64> {
    use crate::domain_computation::primary_graph::output_lineage::invalidation::arc_bytes;
    arc_bytes::<worth_execution::ExecutionArray<T>>()?
        .checked_add(u64::try_from(std::mem::size_of::<T>().checked_mul(count)?).ok()?)
}

// Original/decoded construction is explicitly cold/full. Warm native patch
// never enters this loop; changed native leaves cache the same bound locally.
fn resident_bytes(facts: &[Fact], backing: Option<u64>) -> Option<u64> {
    use worth_execution::ChargedBytes;
    facts.iter().try_fold(backing?, |bytes, fact| {
        crate::domain_computation::primary_graph::fact_verification_observation::original_payload_size();
        bytes.checked_add(fact.additional_charged_bytes())
    })
}
