//! The roster shares original immutable definitions; no definition heap is copied.
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::indexes::data::DerivedIndexDefinition;
use crate::runtime::CapturedIndexDefinitionCatalog;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
struct Backing {
    definitions: Vec<crate::indexes::RetainedDerivedIndexDefinition>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone, Default)]
pub(crate) struct CheckpointIndexDefinitions(Option<Arc<Backing>>);
impl CheckpointIndexDefinitions {
    pub(crate) fn capture_for_request(
        source: &CapturedIndexDefinitionCatalog,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        let mut count = 0usize;
        for _ in source.introduced(1) {
            crate::execution::check_request(request)?;
            count = count.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
        }
        if count == 0 {
            crate::execution::check_request(request)?;
            return Ok(Self::default());
        }
        let control = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Backing>())
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .0
            .pad_to_align()
            .size();
        let roster = Layout::array::<crate::indexes::RetainedDerivedIndexDefinition>(count)
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .size();
        let bytes = control
            .checked_add(roster)
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        let mut definitions = Vec::with_capacity(count);
        for definition in source.introduced(1) {
            crate::execution::check_request(request)?;
            definitions.push(definition.clone());
        }
        Ok(Self(Some(Arc::new(Backing {
            definitions,
            _memory: memory,
        }))))
    }
    /// Independent legacy projection; its new nested heaps are ordinary.
    pub(crate) fn ordinary_owned_copy(&self) -> Vec<DerivedIndexDefinition> {
        self.iter().map(|value| value.as_ref().clone()).collect()
    }
    pub(crate) fn from_ordinary(definitions: Vec<DerivedIndexDefinition>) -> Self {
        if definitions.is_empty() {
            return Self::default();
        }
        Self(Some(Arc::new(Backing {
            definitions: definitions
                .into_iter()
                .map(crate::indexes::RetainedDerivedIndexDefinition::from_owned)
                .collect(),
            _memory: None,
        })))
    }
}
impl std::ops::Deref for CheckpointIndexDefinitions {
    type Target = [crate::indexes::RetainedDerivedIndexDefinition];
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().map_or(&[], |owner| &owner.definitions)
    }
}
impl PartialEq for CheckpointIndexDefinitions {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
impl Eq for CheckpointIndexDefinitions {}
impl Serialize for CheckpointIndexDefinitions {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.deref_slice().serialize(serializer)
    }
}
impl CheckpointIndexDefinitions {
    fn deref_slice(&self) -> &[crate::indexes::RetainedDerivedIndexDefinition] {
        self
    }
}
impl<'de> Deserialize<'de> for CheckpointIndexDefinitions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<DerivedIndexDefinition>::deserialize(deserializer).map(Self::from_ordinary)
    }
}
impl<'a> IntoIterator for &'a CheckpointIndexDefinitions {
    type Item = &'a crate::indexes::RetainedDerivedIndexDefinition;
    type IntoIter = std::slice::Iter<'a, crate::indexes::RetainedDerivedIndexDefinition>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
