//! Signal-owned source-point transport uses its complete declared bound.

use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_signal::facade::durability::{SignalDurableBranchDescriptor, SignalRecordedSourcePoint};
#[test]
fn complete_source_point_fits_the_owned_transport_bound() {
    let owner = DurableSignalOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([7; 16]).unwrap(),
        SignalOwnerSlotIdentity::from_recorded(1),
    );
    let branch = DurableSignalBranchIdentity::from_recorded(owner, 1);
    let point = SignalRecordedSourcePoint::from_recorded(
        SignalBranchAdvanceReference::from_recorded(
            SignalBranchAdvanceIdentity::from_recorded(owner, 1),
            [1; 32],
        ),
        std::num::NonZeroU64::new(1).unwrap(),
        SignalDurableBranchDescriptor::from_recorded(
            DurableSignalGeneration::from_recorded(
                DurableSignalCellIncarnation::from_recorded(branch, 1),
                1,
            ),
            Some(DurableSignalSnapshotIdentity::from_recorded(owner, 1)),
            DurableSignalAdmissionIdentity::from_recorded(owner, 1),
            SignalDefinitionTransitionReference::from_recorded(
                SignalDefinitionTransitionIdentity::from_recorded(owner, 1),
                [2; 32],
            ),
        )
        .unwrap(),
    )
    .unwrap();
    let policy = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&policy);
    let encoded = point.encode_bytes(&request).unwrap();
    assert_eq!(
        encoded.as_ref().len(),
        SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES
    );
    assert_eq!(
        SignalRecordedSourcePoint::readmit_bytes(&request, encoded.as_ref()).unwrap(),
        point
    );
}
