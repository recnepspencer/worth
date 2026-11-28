//! Pure World image/graph admission; no physical authenticity or Native authority.
use super::*;
use crate::durability::{WorldReadmissionDenial as Denial, WorldRecord};
use sha2::Digest;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::WorldDurableJournal;
mod records;

#[test]
fn native_only_image_and_journal_readmission_preserve_cancellation() {
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    let policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    );
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_policy(&policy, cancellation.token(), None);
    assert!(serial.memory().is_none());
    let request = ExecutionRequest::serial(&serial);
    let (image, _) = records::published(&request);
    let bytes = image.encode(&request).unwrap();
    let recovered =
        WorldCheckpointImage::decode_selected(bytes.as_ref(), &image.binding, &request).unwrap();
    assert_eq!(recovered.occurrences.len(), 1);
    recovered.validate_open_identities(&request).unwrap();
    cancellation.cancel();
    assert!(matches!(
        WorldCheckpointImage::decode_selected(bytes.as_ref(), &image.binding, &request),
        Err(Denial::Medium(
            crate::durability::WorldDurableMediumDenial::Request(
                worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::Cancelled
                )
            )
        ))
    ));
}

#[test]
fn decoded_owner_description_clones_retain_actual_original_memory_grants() {
    // Intrinsic grant lifetime only, not a whole-heap quote or Native authority.
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let policy =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&policy);
    let original = records::descriptions(&request).unwrap();
    let charged = budget.charged_memory_bytes();
    assert!(charged > 0);
    let cloned = original.clone();
    assert_eq!(budget.charged_memory_bytes(), charged);
    drop(original);
    assert_eq!(budget.charged_memory_bytes(), charged);
    drop(cloned);
    assert_eq!(budget.charged_memory_bytes(), 0);

    let zero = SerialMemoryBudget::new(Some(0));
    let policy = SerialRequest::from_memory(zero.clone(), CancellationSource::new().token(), None);
    let refused = records::descriptions(&ExecutionRequest::serial(&policy));
    assert!(matches!(refused,
        Err(crate::durability::WorldDurableMediumDenial::NativeDescription(cause))
        if matches!(*cause,worth_relational::facade::branch::RelationalRecordedComponentRefusal::Request(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::PolicyMemoryExhausted {admitted:0,..}
        ))
    ));
    assert_eq!(zero.charged_memory_bytes(), 0);
}

#[test]
fn selected_published_image_refuses_a_later_released_custody_tail() {
    let policy = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&policy);
    let (image, open) = records::published(&request);
    let bytes = image.encode(&request).unwrap();
    let mut recovered =
        WorldCheckpointImage::decode_selected(bytes.as_ref(), &image.binding, &request).unwrap();
    assert_eq!(recovered.occurrences.len(), 1);
    let released = WorldRecord::Released {
        association: open.association,
        custody: open.custody,
        relational: None,
    };
    records::admit(&mut recovered, &released, 2, &request).unwrap();
    assert!(matches!(
        recovered.validate_open_identities(&request),
        Err(Denial::ConflictingDisposition)
    ));
}

#[test]
fn reserved_open_commit_cannot_reuse_the_branch_origin_identity() {
    let policy = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&policy);
    let (image, head) = records::origin(&request);
    let mut image = records::selected(image, &request);
    let colliding = records::open(&head, head.commit.ordinal());
    records::admit(
        &mut image,
        &WorldRecord::OpenAttempt(colliding),
        1,
        &request,
    )
    .unwrap();
    assert!(matches!(
        image.validate_open_identities(&request),
        Err(Denial::DuplicateIdentity)
    ));
    let (mut control, head) = records::origin(&request);
    records::admit(
        &mut control,
        &WorldRecord::OpenAttempt(records::open(&head, 2)),
        1,
        &request,
    )
    .unwrap();
    control.validate_open_identities(&request).unwrap();
}

#[test]
fn selected_image_requires_exact_dense_coordinates_and_end_digests() {
    let policy = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&policy);
    let (image, _) = records::published(&request);
    let mut image = records::selected(image, &request);
    image
        .validate_image_coordinates(&image.coverage, &request)
        .unwrap();
    let mut wrong_end = image.coverage.clone();
    wrong_end.occurrences.digest[0] ^= 1;
    assert!(matches!(
        image.validate_image_coordinates(&wrong_end, &request),
        Err(Denial::JournalSourceMismatch)
    ));
    let mut wrong_count = image.coverage.clone();
    wrong_count.custody.position += 1;
    assert!(matches!(
        image.validate_image_coordinates(&wrong_count, &request),
        Err(Denial::JournalSourceMismatch)
    ));
    // Two rows still exist: matching the count cannot conceal a missing position 1.
    image.owner.as_mut().unwrap().position = 3;
    assert!(matches!(
        image.validate_image_coordinates(&image.coverage, &request),
        Err(Denial::JournalSourceMismatch)
    ));
}

#[test]
fn uncovered_tail_requires_the_next_position_and_its_actual_wire_digest() {
    let policy = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&policy);
    let (image, head) = records::origin(&request);
    let mut image = records::selected(image, &request);
    let record = WorldRecord::OpenAttempt(records::open(&head, 2));
    let (bytes, digest) = records::wire(&record, &request);
    assert!(matches!(
        image.admit_journal_record(
            WorldDurableJournal::Custody,
            2,
            digest,
            bytes.as_ref(),
            &request
        ),
        Err(Denial::JournalSourceMismatch)
    ));
    let mut wrong_digest = digest;
    wrong_digest[0] ^= 1;
    assert!(matches!(
        image.admit_journal_record(
            WorldDurableJournal::Custody,
            1,
            wrong_digest,
            bytes.as_ref(),
            &request
        ),
        Err(Denial::JournalSourceMismatch)
    ));
    image
        .admit_journal_record(
            WorldDurableJournal::Custody,
            1,
            digest,
            bytes.as_ref(),
            &request,
        )
        .unwrap();
    image.validate_open_identities(&request).unwrap();
    image
        .validate_image_coordinates(&image.coverage, &request)
        .unwrap();
}

#[test]
fn selected_checkpoint_rebuilds_expected_reclaim_only_for_unsettled_opens() {
    use crate::durability::ExpectedReclaim;
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let (mut image, head) = records::origin(&request);
    let open = records::open(&head, 2);
    records::admit(
        &mut image,
        &WorldRecord::OpenAttempt(open.clone()),
        1,
        &request,
    )
    .unwrap();

    let mut image = records::selected(image, &request);
    let attempt = open.association.attempt();
    assert!(matches!(
        image.reclaim_expected(&image.opens[&attempt]),
        ExpectedReclaim::Pending(expected) if *expected == head
    ));
    let release = WorldRecord::Released {
        association: open.association,
        custody: open.custody,
        relational: None,
    };
    records::admit(&mut image, &release, 2, &request).unwrap();
    let mut image = records::selected(image, &request);
    assert!(matches!(
        image.reclaim_expected(&image.opens[&attempt]),
        ExpectedReclaim::Released(_)
    ));
    assert!(matches!(
        records::admit(&mut image, &release, 3, &request),
        Err(Denial::ConflictingDisposition)
    ));
}

#[test]
fn world_artifact_rejects_the_reset_to_zero_format() {
    use crate::durability::{WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Medium};
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let (_, head) = records::origin(&request);
    let (bytes, _) = records::wire(&WorldRecord::OpenAttempt(records::open(&head, 2)), &request);
    let mut bytes = bytes.as_ref().to_vec();
    assert_eq!(&bytes[8..12], &6u32.to_le_bytes());
    bytes[8..12].copy_from_slice(&5u32.to_le_bytes());
    assert!(matches!(
        crate::durability::codec::decode(&bytes, head.branch.owner().home(), &request),
        Err(Medium::Frame(Frame::UnsupportedVersion(_)))
    ));
}
