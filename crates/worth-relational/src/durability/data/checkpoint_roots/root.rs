use super::super::RelationalCheckpointRegion;
use crate::branch::RelationalBranchBasisDescriptor;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::history::data::CommitId;
use serde::{Deserialize, Serialize};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RootData {
    pub(crate) commit_id: CommitId,
    pub(crate) recorded_components: Vec<RelationalBranchBasisDescriptor>,
    pub(crate) schema_carrier_digest: [u8; 32],
    pub(crate) regions: Vec<RelationalCheckpointRegion>,
    pub(crate) content: Vec<super::super::RelationalCheckpointContent>,
    #[serde(skip)]
    _memory: Option<ExecutionMemoryReservation>,
    #[serde(skip)]
    _content_memory: Option<ExecutionMemoryReservation>,
}

/// Cloning a root description shares only this root's own immutable allocations.
#[derive(Clone)]
pub(crate) struct RelationalCheckpointRoot(Arc<RootData>);
impl std::ops::Deref for RelationalCheckpointRoot {
    type Target = RootData;
    fn deref(&self) -> &RootData {
        &self.0
    }
}
impl std::fmt::Debug for RelationalCheckpointRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalCheckpointRoot")
            .field("commit_id", &self.commit_id)
            .field("recorded_components", &self.recorded_components)
            .field("schema_carrier_digest", &self.schema_carrier_digest)
            .field("regions", &self.regions)
            .field("content", &self.content)
            .finish()
    }
}
impl PartialEq for RelationalCheckpointRoot {
    fn eq(&self, other: &Self) -> bool {
        self.commit_id == other.commit_id
            && self.recorded_components == other.recorded_components
            && self.schema_carrier_digest == other.schema_carrier_digest
            && self.regions == other.regions
            && self.content == other.content
    }
}
impl Eq for RelationalCheckpointRoot {}
impl Serialize for RelationalCheckpointRoot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.as_ref().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for RelationalCheckpointRoot {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Existing ordinary decoding; controlled source admission is explicit.
        RootData::deserialize(deserializer).map(|data| Self(Arc::new(data)))
    }
}
impl RelationalCheckpointRoot {
    pub(crate) fn from_ordinary_parts(
        commit_id: CommitId,
        recorded_components: Vec<RelationalBranchBasisDescriptor>,
        schema_carrier_digest: [u8; 32],
        regions: Vec<RelationalCheckpointRegion>,
        content: Vec<super::super::RelationalCheckpointContent>,
    ) -> Self {
        Self(Arc::new(RootData {
            commit_id,
            recorded_components,
            schema_carrier_digest,
            regions,
            content,
            _memory: None,
            _content_memory: None,
        }))
    }
}

/// All new nested copies and root control are reserved before either Vec grows.
pub(crate) struct PreparedCheckpointRoot {
    commit_id: CommitId,
    components: Vec<RelationalBranchBasisDescriptor>,
    regions: Vec<RelationalCheckpointRegion>,
    expected_regions: usize,
    memory: Option<ExecutionMemoryReservation>,
}
impl PreparedCheckpointRoot {
    pub(crate) fn prepare_from<'a, I: Iterator<Item = &'a RelationalBranchBasisDescriptor>>(
        commit_id: CommitId,
        component_count: usize,
        components: impl Fn() -> I,
        region_count: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        let mut bytes = control_bytes::<RootData>()?
            .checked_add(array_bytes::<RelationalBranchBasisDescriptor>(
                component_count,
            )?)
            .and_then(|bytes| {
                bytes.checked_add(array_bytes::<RelationalCheckpointRegion>(region_count).ok()?)
            })
            .ok_or(Cause::ChargedBytesOverflow)?;
        for component in components() {
            let parents = component
                .reference()
                .target()
                .as_basis()
                .map_or(0, |target| target.parent_commit_ids().len());
            bytes = bytes
                .checked_add(component.branch_id().0.len() as u64)
                .and_then(|bytes| {
                    bytes.checked_add(component.reference().branch_id().as_str().len() as u64)
                })
                .and_then(|bytes| bytes.checked_add(array_bytes::<u64>(parents).ok()?))
                .ok_or(Cause::ChargedBytesOverflow)?;
        }
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        let mut copied = Vec::with_capacity(component_count);
        for component in components() {
            if copied.len() == component_count {
                return Err(Cause::ChargedBytesOverflow);
            }
            copied.push(component.clone());
        }
        if copied.len() != component_count {
            return Err(Cause::ChargedBytesOverflow);
        }
        Ok(Self {
            commit_id,
            components: copied,
            regions: Vec::with_capacity(region_count),
            expected_regions: region_count,
            memory,
        })
    }
    pub(crate) fn push(&mut self, region: RelationalCheckpointRegion) -> Result<(), Cause> {
        if self.regions.len() == self.expected_regions {
            return Err(Cause::ChargedBytesOverflow);
        }
        self.regions.push(region);
        Ok(())
    }
    pub(crate) fn finish(
        self,
        schema_carrier_digest: [u8; 32],
        content: crate::durability::checkpoints::content::root_capture::CapturedRootContent,
    ) -> Result<RelationalCheckpointRoot, Cause> {
        if self.regions.len() != self.expected_regions {
            return Err(Cause::ChargedBytesOverflow);
        }
        Ok(RelationalCheckpointRoot(Arc::new(RootData {
            commit_id: self.commit_id,
            recorded_components: self.components,
            schema_carrier_digest,
            regions: self.regions,
            content: content.rows,
            _memory: self.memory,
            _content_memory: content.memory,
        })))
    }
}
pub(super) fn control_bytes<T>() -> Result<u64, Cause> {
    Layout::new::<[usize; 2]>()
        .extend(Layout::new::<T>())
        .ok()
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
        .ok_or(Cause::ChargedBytesOverflow)
}
pub(super) fn array_bytes<T>(count: usize) -> Result<u64, Cause> {
    Layout::array::<T>(count)
        .ok()
        .and_then(|layout| u64::try_from(layout.size()).ok())
        .ok_or(Cause::ChargedBytesOverflow)
}
