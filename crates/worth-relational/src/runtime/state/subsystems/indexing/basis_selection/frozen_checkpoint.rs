//! Complete emission freezes exact-root selections as owner Arcs, then resolves portable rows.
use super::{BasisSelections, PreparedIndexBasisSelection};
use crate::durability::checkpoint_history::capture::array_capture_stop;
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::indexes::data::{
    DerivedIndexGeneration, DerivedIndexGenerationId, DerivedIndexId, IndexBasisCheckpoint,
};
use std::alloc::Layout;
use std::cmp::Ordering;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
};

/// Branch selections in ascending `(branch, version)` order, frozen with their generations.
pub(in crate::runtime::state::subsystems::indexing) struct FrozenBasisSelections(
    ExecutionArray<Arc<PreparedIndexBasisSelection>>,
);
/// Portable rows in ascending `(branch, version)` order; bindings ascend by index.
pub(in crate::runtime::state::subsystems::indexing) struct OriginalCheckpointBases {
    rows: ExecutionArray<IndexBasisCheckpoint>,
    _heap: ExecutionArray<Option<ExecutionMemoryReservation>>,
}
impl OriginalCheckpointBases {
    pub(in crate::runtime::state::subsystems::indexing) fn rows(&self) -> &[IndexBasisCheckpoint] {
        &self.rows
    }
}

impl BasisSelections {
    /// Branch selections in ascending `(branch, version)` order. A global lookup is
    /// re-derived from its branch selection when a reader installs it.
    fn branch_selections(&self) -> impl Iterator<Item = &Arc<PreparedIndexBasisSelection>> {
        self.entries
            .iter()
            .filter(|(key, _)| key.0.is_some())
            .map(|(_, basis)| basis)
    }
    pub(in crate::runtime::state::subsystems::indexing) fn freeze_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<FrozenBasisSelections, Denial> {
        let mut count = 0_usize;
        for _ in self.branch_selections() {
            consult(request)?;
            count += 1;
        }
        let mut frozen =
            ExecutionRequestArrayBuilder::allocate(*request, count).map_err(array_capture_stop)?;
        for basis in self.branch_selections() {
            consult(request)?;
            frozen.push(Arc::clone(basis)).map_err(array_capture_stop)?;
        }
        Ok(FrozenBasisSelections(
            frozen.seal().map_err(array_capture_stop)?,
        ))
    }
}

impl FrozenBasisSelections {
    /// A selection owns its payload: one may outlive that publication's
    /// retirement from the generation catalog, and the image must still carry it.
    pub(in crate::runtime::state::subsystems::indexing) fn bound(
        &self,
    ) -> impl Iterator<Item = &Arc<DerivedIndexGeneration>> {
        bound(&self.0)
    }
    /// Whether `roster`, ascending by `(index, generation)`, carries every bound payload.
    pub(in crate::runtime::state::subsystems::indexing) fn rostered(
        &self,
        roster: &[Arc<DerivedIndexGeneration>],
    ) -> bool {
        rostered(&self.0, roster)
    }
    /// `image` is the validated roster of a prior image this owner never admitted.
    /// A live selection keeps its own bindings; the image fills only what it lacks.
    pub(in crate::runtime::state::subsystems::indexing) fn resolve(
        self,
        image: &[IndexBasisCheckpoint],
        roster: &[Arc<DerivedIndexGeneration>],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<OriginalCheckpointBases, Denial> {
        let mut count = 0_usize;
        for _ in Paired::new(&self.0, image) {
            consult(request)?;
            count += 1;
        }
        let mut rows =
            ExecutionRequestArrayBuilder::allocate(*request, count).map_err(array_capture_stop)?;
        let mut heap =
            ExecutionRequestArrayBuilder::allocate(*request, count).map_err(array_capture_stop)?;
        for (live, image) in Paired::new(&self.0, image) {
            consult(request)?;
            let image = admitted_image(live, image);
            let funding = request
                .reserve_retained_memory(row_bytes(live, image)?)
                .map_err(|cause| Denial::Request(cause.into()))?;
            rows.push(portable(live, image, roster)?)
                .map_err(array_capture_stop)?;
            heap.push(funding).map_err(array_capture_stop)?;
        }
        Ok(OriginalCheckpointBases {
            rows: rows.seal().map_err(array_capture_stop)?,
            _heap: heap.seal().map_err(array_capture_stop)?,
        })
    }
}

/// Two ascending rosters visited once, pairing rows that name the same exact root key.
struct Paired<'a> {
    live: &'a [Arc<PreparedIndexBasisSelection>],
    image: &'a [IndexBasisCheckpoint],
}
impl<'a> Paired<'a> {
    fn new(
        live: &'a [Arc<PreparedIndexBasisSelection>],
        image: &'a [IndexBasisCheckpoint],
    ) -> Self {
        Self { live, image }
    }
}
impl<'a> Iterator for Paired<'a> {
    type Item = (
        Option<&'a PreparedIndexBasisSelection>,
        Option<&'a IndexBasisCheckpoint>,
    );
    fn next(&mut self) -> Option<Self::Item> {
        let (live, image) = (self.live, self.image);
        let order = match (live.first(), image.first()) {
            (None, None) => return None,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(live), Some(image)) => {
                (&live.branch, live.version).cmp(&(&image.branch, image.version))
            }
        };
        let mut pair = (None, None);
        if order != Ordering::Greater {
            let (first, rest) = live.split_first()?;
            self.live = rest;
            pair.0 = Some(first.as_ref());
        }
        if order != Ordering::Less {
            let (first, rest) = image.split_first()?;
            self.image = rest;
            pair.1 = Some(first);
        }
        Some(pair)
    }
}

fn bound(
    live: &[Arc<PreparedIndexBasisSelection>],
) -> impl Iterator<Item = &Arc<DerivedIndexGeneration>> {
    live.iter().flat_map(|basis| basis.state.entries.values())
}
fn rostered(
    live: &[Arc<PreparedIndexBasisSelection>],
    roster: &[Arc<DerivedIndexGeneration>],
) -> bool {
    bound(live).all(|generation| {
        require_rostered(roster, generation.index_id, generation.generation_id).is_ok()
    })
}
/// The live owner decides its key: an image row for another commit or schema is stale.
fn admitted_image<'a>(
    live: Option<&PreparedIndexBasisSelection>,
    image: Option<&'a IndexBasisCheckpoint>,
) -> Option<&'a IndexBasisCheckpoint> {
    image.filter(|image| {
        live.is_none_or(|live| live.commit == image.commit && live.schema == image.schema)
    })
}
/// The binding and empty-selection capacities a row is built with.
fn capacities(
    live: Option<&PreparedIndexBasisSelection>,
    image: Option<&IndexBasisCheckpoint>,
) -> Result<(usize, usize), Denial> {
    let (entries, selected_empty) = live.map_or((0, 0), |live| {
        (live.state.entries.len(), live.state.empty.len())
    });
    Ok((
        entries
            .checked_add(image.map_or(0, |image| image.bindings.len()))
            .ok_or_else(overflow)?,
        selected_empty
            .checked_add(image.map_or(0, |image| image.empty.len()))
            .ok_or_else(overflow)?,
    ))
}
/// The retained heap of one portable row, quoted before it is built.
fn row_bytes(
    live: Option<&PreparedIndexBasisSelection>,
    image: Option<&IndexBasisCheckpoint>,
) -> Result<u64, Denial> {
    let branch = live
        .map(|live| &live.branch)
        .or_else(|| image.map(|image| &image.branch))
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    let (binding_capacity, empty_capacity) = capacities(live, image)?;
    Layout::array::<(DerivedIndexId, DerivedIndexGenerationId)>(binding_capacity)
        .ok()
        .zip(Layout::array::<DerivedIndexId>(empty_capacity).ok())
        .and_then(|(bindings, empty)| bindings.size().checked_add(empty.size()))
        .and_then(|bytes| bytes.checked_add(branch.0.len()))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(overflow)
}
/// One exact-root row: live bindings win per index, the admitted image fills the rest.
fn portable(
    live: Option<&PreparedIndexBasisSelection>,
    image: Option<&IndexBasisCheckpoint>,
    roster: &[Arc<DerivedIndexGeneration>],
) -> Result<IndexBasisCheckpoint, Denial> {
    let (branch, commit, version, schema) = live
        .map(|live| (&live.branch, live.commit, live.version, live.schema))
        .or_else(|| image.map(|image| (&image.branch, image.commit, image.version, image.schema)))
        .ok_or(Denial::OriginalAuthorityUnavailable)?;
    // Exactly the capacities `row_bytes` funded: the row never grows past them.
    let (binding_capacity, empty_capacity) = capacities(live, image)?;
    let mut bindings = Vec::with_capacity(binding_capacity);
    let mut empty = Vec::with_capacity(empty_capacity);
    if let Some(live) = live {
        for (index, generation) in &live.state.entries {
            require_rostered(roster, *index, generation.generation_id)?;
            bindings.push((*index, generation.generation_id));
        }
        empty.extend(live.state.empty.iter().copied());
    }
    if let Some(image) = image {
        let selected = |index: &DerivedIndexId| {
            live.is_some_and(|live| {
                live.state.entries.contains_key(index) || live.state.empty.contains(index)
            })
        };
        for (index, generation) in image.bindings.iter().filter(|(index, _)| !selected(index)) {
            require_rostered(roster, *index, *generation)?;
            bindings.push((*index, *generation));
        }
        empty.extend(image.empty.iter().filter(|index| !selected(index)).copied());
    }
    bindings.sort_unstable_by_key(|(index, _)| *index);
    empty.sort_unstable();
    Ok(IndexBasisCheckpoint {
        branch: branch.clone(),
        commit,
        version,
        schema,
        empty,
        bindings,
    })
}

/// A binding whose payload is not emitted would make the written image unreadable.
fn require_rostered(
    roster: &[Arc<DerivedIndexGeneration>],
    index: DerivedIndexId,
    generation: DerivedIndexGenerationId,
) -> Result<(), Denial> {
    roster
        .binary_search_by_key(&(index, generation), |row| {
            (row.index_id, row.generation_id)
        })
        .map(|_| ())
        .map_err(|_| Denial::OriginalAuthorityUnavailable)
}
fn consult(request: &ExecutionRequest<'_, '_>) -> Result<(), Denial> {
    request
        .consult()
        .map_err(|cause| Denial::Request(cause.into()))
}
fn overflow() -> Denial {
    Denial::Request(Cause::MemoryOverflow)
}

#[cfg(test)]
mod tests;
