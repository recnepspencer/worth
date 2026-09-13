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

fn adjust_entity_pin(
    runtime: &RelationalRuntime,
    entity_id: crate::identity::data::EntityId,
    class: SubstratePinClass,
    delta: i32,
) {
    adjust_record_pin::<EntityRecordKind>(runtime, entity_id, class, delta);
}

fn adjust_relation_pin(
    runtime: &RelationalRuntime,
    relation_id: crate::identity::data::RelationId,
    class: SubstratePinClass,
    delta: i32,
) {
    adjust_record_pin::<RelationRecordKind>(runtime, relation_id, class, delta);
}

fn adjust_record_pin<K: RecordKind>(
    runtime: &RelationalRuntime,
    record_id: crate::identity::data::RecordId<K::Domain>,
    class: SubstratePinClass,
    delta: i32,
) {
    let retention_fence = runtime
        .visibility
        .retention_fence_version(runtime.current_version_id());
    runtime
        .storage_authority()
        .adjust_named_pin::<K>(record_id, class, delta, retention_fence);
}

impl RelationalRuntime {
    pub(crate) fn visibility_pins(&self) -> VisibilityPinAuthority<'_> {
        VisibilityPinAuthority::new(self)
    }
}
