mod branch_head_residency;
mod replay_pins;

use crate::runtime::RelationalRuntime;
use crate::storage::substrate::PinClass as SubstratePinClass;
use crate::storage::substrate::{EntityRecordKind, RecordKind, RelationRecordKind};
use crate::visibility::snapshot_states::SnapshotState;

pub(crate) struct VisibilityPinAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl<'runtime> VisibilityPinAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

fn adjust_record_pin<K: RecordKind>(
    runtime: &RelationalRuntime,
    residency: &crate::storage::residency::RelationalResidencyLease<
        crate::storage::residency::Full,
    >,
    record_id: crate::identity::data::RecordId<K::Domain>,
    class: SubstratePinClass,
    delta: i32,
) -> Result<(), crate::storage::data::RelationalStorageUnavailable> {
    let retention_fence = runtime
        .visibility
        .retention_fence_version(runtime.current_version_id());
    runtime.storage_authority().adjust_named_pin::<K>(
        residency,
        record_id,
        class,
        delta,
        retention_fence,
    )
}

impl RelationalRuntime {
    pub(crate) fn visibility_pins(&self) -> VisibilityPinAuthority<'_> {
        VisibilityPinAuthority::new(self)
    }
}
