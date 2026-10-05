//! Work for semantic copies and an Arc copy-on-write lineage stamp.
use super::{RetainedStoragePreparation, RuntimeArtifactState, SignalError};
use crate::data::retained_storage::{RetainedStorageCharge as Charge, RetainedStorageMeasurement};
use std::sync::Arc;

pub(super) fn checkpoint_copy(
    work: &mut RetainedStoragePreparation<'_>,
    bytes: u64,
) -> Result<(), SignalError> {
    work.checkpoint_request_only(
        usize::try_from(bytes)
            .map_err(|_| SignalError::invalid_input("diagnostic copy work overflow"))?,
    )
    .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
}

pub(super) fn checkpoint_runtime_detach(
    runtime: Option<&Arc<RuntimeArtifactState>>,
    work: &mut RetainedStoragePreparation<'_>,
) -> Result<(), SignalError> {
    let Some(runtime) = runtime else {
        return Ok(());
    };
    let shared = Arc::strong_count(runtime) > 1;
    if !shared && Arc::weak_count(runtime) == 0 {
        return Ok(());
    }
    let cell = Charge::capacity::<RuntimeArtifactState>(1)
        .and_then(|charge| charge.checked_add(Charge::capacity::<usize>(2)?))
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
    let heap = if shared {
        runtime
            .as_ref()
            .retained_heap_charge(work)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?
    } else {
        Charge::ZERO
    };
    let bytes = cell
        .checked_add(heap)
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?
        .bytes();
    checkpoint_copy(work, bytes)
}
