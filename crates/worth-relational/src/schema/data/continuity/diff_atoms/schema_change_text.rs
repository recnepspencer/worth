//! Intrinsic source admission follows this same private shared text owner.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionMemoryReservation;

pub struct SchemaChangeText(Arc<Backing>);
struct Backing {
    value: Arc<str>,
    memory: OnceLock<Option<ExecutionMemoryReservation>>,
}
impl SchemaChangeText {
    pub fn as_ref(&self) -> &str {
        &self.0.value
    }
    pub(crate) fn control_bytes() -> Option<u64> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<Backing>())
            .ok()?
            .0
            .pad_to_align();
        u64::try_from(layout.size()).ok()
    }
    pub(crate) fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.memory.set(memory)
    }
}
impl From<Arc<str>> for SchemaChangeText {
    fn from(value: Arc<str>) -> Self {
        Self(Arc::new(Backing {
            value,
            memory: OnceLock::new(),
        }))
    }
}
impl Clone for SchemaChangeText {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl std::ops::Deref for SchemaChangeText {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_ref()
    }
}
impl AsRef<str> for SchemaChangeText {
    fn as_ref(&self) -> &str {
        &self.0.value
    }
}
impl std::fmt::Debug for SchemaChangeText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.value.fmt(f)
    }
}
impl PartialEq for SchemaChangeText {
    fn eq(&self, other: &Self) -> bool {
        self.0.value == other.0.value
    }
}
impl Eq for SchemaChangeText {}
impl PartialOrd for SchemaChangeText {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SchemaChangeText {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.value.cmp(&other.0.value)
    }
}
impl std::hash::Hash for SchemaChangeText {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.0.value, state);
    }
}
impl Serialize for SchemaChangeText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.value.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for SchemaChangeText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Arc::<str>::deserialize(deserializer).map(Self::from)
    }
}
impl From<&str> for SchemaChangeText {
    fn from(value: &str) -> Self {
        Self::from(Arc::<str>::from(value))
    }
}
impl From<String> for SchemaChangeText {
    fn from(value: String) -> Self {
        Self::from(Arc::<str>::from(value))
    }
}
impl std::fmt::Display for SchemaChangeText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.value.fmt(f)
    }
}
impl std::borrow::Borrow<str> for SchemaChangeText {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}
