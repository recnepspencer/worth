//! Snapshot issuance, exact operation and release remain one public owner sequence.
use super::pin_read_refusal::pin_denial;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    runtime::RelationalRuntime, transactions::RelationalExecutionDenialCause,
};
pub(super) enum SnapshotRead {
    Read,
    CapabilityPin,
    Inspection,
    PinInspection,
}
pub(super) fn read(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    operation: SnapshotRead,
) -> Result<(), RelationalExecutionDenialCause> {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let handle = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let reader = runtime.read_truth();
    let answer = match operation {
        SnapshotRead::Read => reader
            .read_snapshot(&handle, request)
            .map(|value| assert!(value.is_some(), "actual snapshot resolves")),
        SnapshotRead::CapabilityPin => reader
            .pin_read_capability(&handle, &[], request)
            .map(|_| ())
            .map_err(pin_denial),
        SnapshotRead::Inspection => reader
            .inspect_snapshot(request, &handle)
            .map(|value| assert!(value.is_some(), "actual snapshot resolves")),
        SnapshotRead::PinInspection => runtime
            .inspect_what_happened()
            .inspect_snapshot_pinning(request, &handle)
            .map(|value| assert!(value.is_some(), "actual snapshot resolves")),
    };
    runtime.snapshots().release_snapshot(&handle).unwrap();
    answer
}
