//! Emit required original component roots from independently reconstructed Full DATA.
use worth_foundational::CanonicalDigestId;
mod covered;
mod emission;
mod grouping;
pub(super) enum CheckpointMirror<'a> {
    CurrentMainRoot,
    Substrate {
        edition: &'a crate::runtime::PartitionEdition,
        schema: &'a crate::branch::RelationalBranchRootSchemaAuthority,
        carrier: CanonicalDigestId,
    },
}
use super::{branches::BranchImages, roots::CapturedRoots, schemas::CompleteSchemas};
use crate::branch::{RelationalBranchBasisDescriptor, RelationalBranchRoot};
use crate::durability::{data::*, medium::RelationalDurableMemberLocator};
use crate::history::data::BranchId;
use crate::storage::residency::{RelationalCheckpointRegionPublication, RelationalRegionResidency};
use crate::symbols::data::SymbolTextLookup;
use grouping::Grouped;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest,
};
use RelationalNativeCheckpointCaptureDenial as Denial;

pub(super) struct RegionOutput {
    roots: CheckpointRoots,
    empty: Vec<RelationalCheckpointEmptyComponent>,
    mirror: Vec<RelationalCheckpointRegion>,
    mirror_schema: Option<CanonicalDigestId>,
    publications: ExecutionArray<RelationalCheckpointRegionPublication>,
    cells: ExecutionArray<Arc<RelationalRegionResidency>>,
    _empty_memory: Option<ExecutionMemoryReservation>,
    _mirror_memory: Option<ExecutionMemoryReservation>,
}
pub(super) struct RegionCaptureCustody {
    pub(super) publications: ExecutionArray<RelationalCheckpointRegionPublication>,
    pub(super) cells: ExecutionArray<Arc<RelationalRegionResidency>>,
}
impl RegionOutput {
    /// One closed transfer into the same fresh controlled draft; no raw ticket
    /// or bare Vec extraction is exposed to a caller.
    pub(super) fn install_into(
        self,
        draft: &mut RelationalCheckpointManifestDraft,
    ) -> RegionCaptureCustody {
        draft.roots = self.roots;
        draft.retained_empty_components = self.empty;
        draft.mirror = self.mirror;
        draft.mirror_schema = self.mirror_schema.map(|digest| *digest.bytes());
        draft.allocations.empty_components = self._empty_memory;
        draft.allocations.mirror = self._mirror_memory;
        RegionCaptureCustody {
            publications: self.publications,
            cells: self.cells,
        }
    }
}
pub(super) fn capture<
    'a,
    I: Iterator<
        Item = (
            &'a RelationalBranchBasisDescriptor,
            &'a Arc<RelationalBranchRoot>,
        ),
    >,
>(
    observations: impl Fn() -> I,
    main: &BranchId,
    branches: &BranchImages<'_>,
    roots: &CapturedRoots,
    base: Option<&crate::durability::checkpoint_admission::FrozenSelectedCaptureBase>,
    schemas: &CompleteSchemas<'_>,
    dictionary: &dyn SymbolTextLookup,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RegionOutput, Denial> {
    let grouped = Grouped::capture(observations, main, branches, roots, schemas, request)?;
    emission::capture(
        grouped,
        roots,
        base,
        CheckpointMirror::CurrentMainRoot,
        dictionary,
        members,
        request,
    )
}
pub(super) fn capture_interim<
    'a,
    I: Iterator<
        Item = (
            &'a RelationalBranchBasisDescriptor,
            &'a Arc<RelationalBranchRoot>,
        ),
    >,
>(
    observations: impl Fn() -> I,
    main: &BranchId,
    branches: &BranchImages<'_>,
    roots: &CapturedRoots,
    schemas: &CompleteSchemas<'_>,
    mirror: CheckpointMirror<'_>,
    dictionary: &dyn SymbolTextLookup,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RegionOutput, Denial> {
    let grouped = Grouped::capture(observations, main, branches, roots, schemas, request)?;
    emission::capture(grouped, roots, None, mirror, dictionary, members, request)
}
fn request_stop(cause: crate::execution::RelationalExecutionDenialCause) -> Denial {
    Denial::Request(cause)
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    crate::durability::checkpoint_history::capture::capture_stop(
        crate::durability::checkpoint_history::capture::map_stop(cause),
    )
}
fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Denial {
    crate::durability::checkpoint_history::capture::array_capture_stop(cause)
}
fn overflow() -> Denial {
    request_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
fn reserve(
    bytes: usize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<ExecutionMemoryReservation>, Denial> {
    crate::execution::check_request(request).map_err(request_stop)?;
    if bytes == 0 {
        return Ok(None);
    }
    request
        .reserve_retained_memory(u64::try_from(bytes).map_err(|_| overflow())?)
        .map_err(|cause| request_stop(cause.into()))
}
fn array_bytes<T>(count: usize) -> Result<usize, Denial> {
    std::alloc::Layout::array::<T>(count)
        .map(|layout| layout.size())
        .map_err(|_| overflow())
}
