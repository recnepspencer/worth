//! A custom merge policy name retains its own shared backing admission.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionMemoryReservation;
/// Shared text with borrowed access. Ordinary `From` construction moves its
/// input and allocates an unbudgeted private control cell; it grants no authority.
pub struct MergePolicyName(Arc<Backing>);
struct Backing {
    text: Arc<str>,
    memory: OnceLock<Option<ExecutionMemoryReservation>>,
}
impl MergePolicyName {
    pub fn as_str(&self) -> &str {
        &self.0.text
    }
    pub(crate) fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.memory.set(memory)
    }
    pub(crate) fn control_bytes() -> Option<u64> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<Backing>())
            .ok()?
            .0
            .pad_to_align();
        u64::try_from(layout.size()).ok()
    }
    #[cfg(test)]
    pub(crate) fn storage_bytes(length: usize) -> Option<u64> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::array::<u8>(length).ok()?)
            .ok()?
            .0
            .pad_to_align();
        Self::control_bytes()?.checked_add(u64::try_from(layout.size()).ok()?)
    }
}
impl From<Arc<str>> for MergePolicyName {
    fn from(text: Arc<str>) -> Self {
        Self(Arc::new(Backing {
            text,
            memory: OnceLock::new(),
        }))
    }
}
impl From<&str> for MergePolicyName {
    fn from(text: &str) -> Self {
        Self::from(Arc::<str>::from(text))
    }
}
impl From<String> for MergePolicyName {
    fn from(text: String) -> Self {
        Self::from(Arc::<str>::from(text))
    }
}
impl Clone for MergePolicyName {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl std::ops::Deref for MergePolicyName {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
impl AsRef<str> for MergePolicyName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Debug for MergePolicyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.text.fmt(f)
    }
}
impl std::fmt::Display for MergePolicyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.text.fmt(f)
    }
}
impl PartialEq for MergePolicyName {
    fn eq(&self, other: &Self) -> bool {
        self.0.text == other.0.text
    }
}
impl Eq for MergePolicyName {}
impl PartialOrd for MergePolicyName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for MergePolicyName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.text.cmp(&other.0.text)
    }
}
impl Serialize for MergePolicyName {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.text.serialize(s)
    }
}
impl<'de> Deserialize<'de> for MergePolicyName {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Arc::<str>::deserialize(d).map(Self::from)
    }
}
