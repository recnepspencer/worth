//! The final fixed image roster; its outer Manifest supplies immutable sharing.
use crate::durability::data::DurableBranchRootSchemaImage;
use crate::execution::RelationalExecutionDenialCause as Cause;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest};
#[derive(Debug, Default)]
pub(crate) struct CheckpointSchemaImages {
    images: Vec<DurableBranchRootSchemaImage>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl CheckpointSchemaImages {
    pub(crate) fn from_ordinary(images: Vec<DurableBranchRootSchemaImage>) -> Self {
        Self {
            images,
            _memory: None,
        }
    }
    /// Only this independent pointer roster is copied; image backing stays shared.
    /// This explicit ordinary projection does not claim to fund the new Vec.
    pub(crate) fn ordinary_owned_copy(&self) -> Vec<DurableBranchRootSchemaImage> {
        self.images.clone()
    }
    pub(in crate::durability) fn copy_for_request(
        source: &ExecutionOrderedMap<[u8; 32], DurableBranchRootSchemaImage>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        crate::execution::check_request(request)?;
        let bytes = std::alloc::Layout::array::<DurableBranchRootSchemaImage>(source.len())
            .ok()
            .and_then(|layout| u64::try_from(layout.size()).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        let mut images = Vec::with_capacity(source.len());
        for (_, image) in source.iter() {
            crate::execution::check_request(request)?;
            images.push(image.clone());
        }
        Ok(Self {
            images,
            _memory: memory,
        })
    }
}
impl std::ops::Deref for CheckpointSchemaImages {
    type Target = [DurableBranchRootSchemaImage];
    fn deref(&self) -> &Self::Target {
        &self.images
    }
}
impl PartialEq for CheckpointSchemaImages {
    fn eq(&self, other: &Self) -> bool {
        self.images == other.images
    }
}
impl Eq for CheckpointSchemaImages {}
impl Serialize for CheckpointSchemaImages {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.images.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CheckpointSchemaImages {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<DurableBranchRootSchemaImage>::deserialize(deserializer).map(Self::from_ordinary)
    }
}
impl<'a> IntoIterator for &'a CheckpointSchemaImages {
    type Item = &'a DurableBranchRootSchemaImage;
    type IntoIter = std::slice::Iter<'a, DurableBranchRootSchemaImage>;
    fn into_iter(self) -> Self::IntoIter {
        self.images.iter()
    }
}
