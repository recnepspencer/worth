//! Descriptive index copies retain only their own original admitted storage.
use super::*;
use crate::history::data::{BranchId, CommitId, RelationalAssociationDiscovery};
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
fn input() -> RelationalCommitIndexEntry {
    let mut name = String::with_capacity(16 * 1024);
    name.push_str("short-index");
    let mut parents = Vec::with_capacity(2048);
    parents.push(CommitId(1));
    RelationalCommitIndexEntry {
        association_discovery: RelationalAssociationDiscovery::Unassociated,
        commit: RelationalCommitReceipt {
            commit_id: CommitId(2),
            version_id: crate::identity::data::VersionId(2),
            branch_id: BranchId(name),
            parents,
        },
        position: crate::publication::patch::data::PatchStreamPosition(2),
        root: crate::branch::RelationalBranchRootDescriptor::new([3; 32], [4; 32]),
        payload_digest: [5; 32],
        schema_version: crate::schema::data::SchemaVersionId(7),
    }
}
fn bill(entry: &RelationalCommitIndexEntry) -> u64 {
    // Independent concrete final backing layout plus actual copied lengths.
    let control = Layout::new::<(usize, usize)>()
        .extend(Layout::new::<Backing>())
        .unwrap()
        .0
        .pad_to_align()
        .size();
    let parents = Layout::array::<CommitId>(entry.commit.parents.len())
        .unwrap()
        .size();
    u64::try_from(control + parents + entry.commit.branch_id.0.len()).unwrap()
}
fn host(limit: u64) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(limit)),
        CancellationSource::new().token(),
        None,
    )
}
#[test]
fn retained_index_clone_keeps_exact_metadata_after_original_drop() {
    let input = input();
    println!(
        "retained-index layouts entry={} backing={} handle={} envelope-cell={}",
        std::mem::size_of::<RelationalCommitIndexEntry>(),
        std::mem::size_of::<Backing>(),
        std::mem::size_of::<RetainedCanonicalIndex>(),
        std::mem::size_of::<crate::history::RelationalCommitEnvelopeResidency>()
    );
    assert!(input.commit.parents.capacity() > 1024);
    assert!(input.commit.branch_id.0.capacity() > 4096);
    let host = host(64 * 1024);
    let request = ExecutionRequest::serial(&host);
    let original = RetainedCanonicalIndex::copy_original(&input, &request).unwrap();
    assert_eq!(original.as_ref(), &input);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), bill(&input));
    let clone = original.clone();
    assert!(clone.same_owner(&original));
    assert!(std::ptr::eq(clone.as_ref(), original.as_ref()));
    drop(original);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), bill(&input));
    assert_eq!(clone.commit.branch_id.0, "short-index");
    drop(clone);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    assert_eq!(
        std::mem::size_of::<RetainedCanonicalIndex>(),
        std::mem::size_of::<usize>()
    );
}
#[test]
fn retained_index_copy_refuses_exact_bill_before_copy_or_cancellation() {
    let input = input();
    let calibration = host(64 * 1024);
    let request = ExecutionRequest::serial(&calibration);
    let mut controller = None;
    request
        .run_native_with_checkpoint(|_| {
            controller = Some(calibration.memory().unwrap().charged_memory_bytes())
        })
        .unwrap();
    let controller = controller.unwrap();
    assert!(controller > 0);
    let target = host(controller + bill(&input) - 1);
    let request = ExecutionRequest::serial(&target);
    let cause = RetainedCanonicalIndex::copy_original(&input, &request).unwrap_err();
    let (requested, admitted) = match cause {
        crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
            requested,
            admitted,
            ancestor: 0,
        } => (requested, admitted),
        other => panic!("exact entered metadata allocation required: {other:?}"),
    };
    assert_eq!(requested, bill(&input));
    assert_eq!(admitted, bill(&input) - 1);
    assert_eq!(target.memory().unwrap().charged_memory_bytes(), 0);
    let cancel = CancellationSource::new();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(64 * 1024)),
        cancel.token(),
        None,
    );
    cancel.cancel();
    let request = ExecutionRequest::serial(&host);
    assert!(RetainedCanonicalIndex::copy_original(&input, &request).is_err());
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    assert_eq!(input.commit.branch_id.0, "short-index");
    assert_eq!(input.commit.parents, [CommitId(1)]);
}
