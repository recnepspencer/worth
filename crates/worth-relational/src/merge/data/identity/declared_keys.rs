//! Opaque shared declared identity keys keep their original allocation and funding together.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionMemoryReservation;
type Element = worth_foundational::facade::AspectKey;

/// Shared keys with borrowed access. Ordinary `From` construction moves its
/// input and allocates an unbudgeted private control cell; it grants no authority.
pub struct DeclaredIdentityKeys(Arc<Backing>);
struct Backing {
    values: Arc<[Element]>,
    memory: OnceLock<Option<ExecutionMemoryReservation>>,
}
impl DeclaredIdentityKeys {
    pub fn as_slice(&self) -> &[Element] {
        &self.0.values
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
    pub(crate) fn storage_bytes(count: usize) -> Option<u64> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::array::<Element>(count).ok()?)
            .ok()?
            .0
            .pad_to_align();
        Self::control_bytes()?.checked_add(u64::try_from(layout.size()).ok()?)
    }
}
impl From<Arc<[Element]>> for DeclaredIdentityKeys {
    fn from(values: Arc<[Element]>) -> Self {
        Self(Arc::new(Backing {
            values,
            memory: OnceLock::new(),
        }))
    }
}
impl From<Vec<Element>> for DeclaredIdentityKeys {
    fn from(values: Vec<Element>) -> Self {
        Self::from(Arc::<[Element]>::from(values))
    }
}
impl<const N: usize> From<[Element; N]> for DeclaredIdentityKeys {
    fn from(values: [Element; N]) -> Self {
        Self::from(Arc::<[Element]>::from(values))
    }
}
impl Clone for DeclaredIdentityKeys {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl std::ops::Deref for DeclaredIdentityKeys {
    type Target = [Element];
    fn deref(&self) -> &[Element] {
        self.as_slice()
    }
}
impl AsRef<[Element]> for DeclaredIdentityKeys {
    fn as_ref(&self) -> &[Element] {
        self.as_slice()
    }
}
impl<'a> IntoIterator for &'a DeclaredIdentityKeys {
    type Item = &'a Element;
    type IntoIter = std::slice::Iter<'a, Element>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl std::fmt::Debug for DeclaredIdentityKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.values.fmt(f)
    }
}
impl PartialEq for DeclaredIdentityKeys {
    fn eq(&self, other: &Self) -> bool {
        self.0.values == other.0.values
    }
}
impl Eq for DeclaredIdentityKeys {}
impl PartialOrd for DeclaredIdentityKeys {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for DeclaredIdentityKeys {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.values.cmp(&other.0.values)
    }
}
impl Serialize for DeclaredIdentityKeys {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.values.serialize(s)
    }
}
impl<'de> Deserialize<'de> for DeclaredIdentityKeys {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Arc::<[Element]>::deserialize(d).map(Self::from)
    }
}
