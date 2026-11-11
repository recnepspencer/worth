//! One original rule-name backing owns its private grant slot through clones.
use super::*;
use serde::{Deserializer, Serializer};
use worth_execution::ExecutionMemoryReservation;

pub(super) struct Backing {
    text: Arc<str>,
    // Last: name storage releases before its original admission.
    memory: OnceLock<Option<ExecutionMemoryReservation>>,
}
impl Backing {
    pub(super) fn new(text: Arc<str>) -> Self {
        Self {
            text,
            memory: OnceLock::new(),
        }
    }
    pub(super) fn text(&self) -> &str {
        &self.text
    }
    pub(super) fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.memory.set(memory)
    }
    pub(super) fn control_bytes() -> Option<u64> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<Self>())
            .ok()?
            .0
            .pad_to_align();
        u64::try_from(layout.size()).ok()
    }
}
impl Clone for CustomInvariantRuleId {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl std::fmt::Debug for CustomInvariantRuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CustomInvariantRuleId")
            .field(&self.as_str())
            .finish()
    }
}
impl PartialEq for CustomInvariantRuleId {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl Eq for CustomInvariantRuleId {}
impl PartialOrd for CustomInvariantRuleId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CustomInvariantRuleId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}
impl std::hash::Hash for CustomInvariantRuleId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(self.as_str(), state);
    }
}
impl Serialize for CustomInvariantRuleId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct("CustomInvariantRuleId", self.as_str())
    }
}
impl<'de> Deserialize<'de> for CustomInvariantRuleId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Arc::<str>::deserialize(deserializer).map(Self::new)
    }
}
