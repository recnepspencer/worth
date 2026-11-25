//! Emit required original component roots from independently reconstructed Full DATA.
use worth_foundational::CanonicalDigestId;
use worth_relational_contracts::internal_port as contracts;
pub mod covered;
pub mod emission;
pub mod grouping;
use super::{branches::BranchImages, roots::CapturedRoots, schemas::CompleteSchemas};
use crate::durability::data::*;
use contracts::history::data::coordinates::BranchId;
use contracts::symbols::data::symbol_text_lookup::SymbolTextLookup;
use grouping::Grouped;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest,
};
use worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::storage::residency::cell::RelationalRegionResidency;
use worth_relational_storage::internal_port::storage::residency::checkpoint_capture::RelationalCheckpointRegionPublication;
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
    emission::capture(grouped, roots, base, dictionary, members, request)
}
fn request_stop(
    cause: contracts::execution::denial_cause::RelationalExecutionDenialCause,
) -> Denial {
    Denial::Request(cause)
}
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::capture_stop(
        contracts::durability::data::recovery_stop::map_stop(cause),
    )
}
fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop(cause)
}
fn overflow() -> Denial {
    request_stop(
        contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
    )
}
fn reserve(
    bytes: usize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<ExecutionMemoryReservation>, Denial> {
    contracts::execution::request_custody::check_request(request).map_err(request_stop)?;
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
