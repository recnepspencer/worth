//! Actual EOF ownership retires before clean physical close, without reclaim.
use super::*;
use sha2::{Digest, Sha256};

fn verified_read(
    serving: &ServingPhysicalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> (AnchoredRecordReadMembership, RecordReadObservation) {
    let reader = serving.try_records_for_checkpoint(request).unwrap();
    let records = serving.anchored_records().unwrap();
    let mut stream = records
        .open_payload_from(&reader, request, anchor(), 1 << 20)
        .unwrap()
        .unwrap();
    let mut scratch = [0; 64 * 1024];
    while stream.read_next(&mut scratch).unwrap() != 0 {}
    let version = stream.verified_version().expect("actual verified EOF");
    // Independently observe the exact verified record through the SAME frozen
    // reader, rather than infer its placement from the chosen payload length.
    // This ordinary observer is disposed before the bounded retirement claim.
    let physical = reader
        .open(
            version.record(),
            RecordReadLimits::new(RecordByteLimit::new(1 << 20).unwrap()),
        )
        .unwrap();
    let observation = physical.observation();
    drop(physical);
    let membership = stream
        .into_verified_membership(reader)
        .unwrap_or_else(|_| panic!("actual EOF"));
    (membership, observation)
}

#[allow(
    clippy::while_let_loop,
    reason = "Keep every retirement progress fate explicit"
)]
fn finish(
    serving: &ServingPhysicalRuntime,
    request: &ExecutionRequest<'_, '_>,
    mut task: AnchoredRecordReadRetirement,
) -> RetiredAnchoredRecordReadMembership {
    let records = serving.anchored_records().unwrap();
    loop {
        match task.advance(&records, request).unwrap() {
            AnchoredRecordReadRetirementProgress::Advanced => {}
            AnchoredRecordReadRetirementProgress::Complete => break,
        }
    }
    task.into_retired()
        .unwrap_or_else(|_| panic!("completed original task"))
}

fn assert_clean_close(serving: ServingPhysicalRuntime) {
    let closed = serving.close_plan().execute();
    assert!(
        matches!(closed, PhysicalStoreCloseOutcome::Closed { .. }),
        "actual Store classification must be clean"
    );
    let shutdown = closed.shutdown();
    assert_eq!(
        shutdown.read_protection().disposition(),
        PhysicalReadProtectionDisposition::Released
    );
    assert_eq!(
        shutdown.read_protection().observation().live_acquisitions(),
        0
    );
    assert_eq!(shutdown.records().counters().live_handles(), 0);
    let counters = shutdown.residency().counters();
    assert_eq!(counters.pin_leases(), 0);
    assert_eq!(counters.pinned_frames(), 0);
    assert_eq!(counters.active_operation_bytes(), 0);
    assert!(!shutdown.residency().requires_inspection());
}

#[test]
fn verified_inline_and_extent_reads_retire_before_clean_store_close() {
    // The real fixture's placement policy puts the small value inline and the
    // larger value in an extent; both retain a real EOF session and frame.
    for payload_bytes in [128, 150_001] {
        let directory = tempfile::tempdir().unwrap();
        let serving = initialize(directory.path());
        let cancellation = CancellationSource::new();
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(8 << 20)),
            cancellation.token(),
            None,
        );
        let request = ExecutionRequest::serial(&serial);
        let member = b"original selected member".to_vec();
        let selected = serving
            .anchored_records()
            .unwrap()
            .publish(
                AnchoredRecordPublication::new(
                    anchor(),
                    0,
                    vec![29; payload_bytes],
                    fixture::placement(),
                    deadline(),
                )
                .with_members(vec![AnchoredRecordMember::New {
                    key: Sha256::digest(&member).into(),
                    bytes: member,
                }]),
            )
            .unwrap();
        let (membership, observation) = verified_read(&serving, &request);
        assert_eq!(
            observation.touched_extents(),
            u64::from(payload_bytes > 128),
            "actual selected record must exercise the intended placement"
        );
        assert_eq!(membership.membership(), selected.membership());
        let retired = finish(&serving, &request, membership.into_retirement());
        assert!(serial.memory().unwrap().charged_memory_bytes() > 0);
        assert_clean_close(serving);
        // Close succeeded while the same original data and admissions remain.
        assert!(serial.memory().unwrap().charged_memory_bytes() > 0);
        retired.reclaim();
        assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
    }
}

#[test]
fn shared_read_retirement_releases_only_the_original_local_share() {
    let directory = tempfile::tempdir().unwrap();
    let serving = initialize(directory.path());
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        cancellation.token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    publish_anchor(&serving, 0, vec![37; 150_001]);
    let (membership, _) = verified_read(&serving, &request);
    let independent = serving.try_records_for_checkpoint(&request).unwrap();
    let protection = serving.read_protection_observer();
    let before = protection.snapshot().live_acquisitions();
    let other = membership.clone();
    let retired_local = finish(&serving, &request, membership.into_retirement());
    assert_eq!(protection.snapshot().live_acquisitions(), before);
    let records = serving.anchored_records().unwrap();
    assert!(records.validate_read_membership(&other).is_ok());
    let retired_original = finish(&serving, &request, other.into_retirement());
    assert_eq!(
        protection.snapshot().live_acquisitions(),
        1,
        "independently acquired actual reader remains protected"
    );
    // This borrowed facade has no custody; its last use ends the loan.
    drop(independent);
    assert_clean_close(serving);
    retired_local.reclaim();
    retired_original.reclaim();
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn foreign_store_and_original_stop_preserve_read_retirement_task() {
    let directory = tempfile::tempdir().unwrap();
    let foreign_directory = tempfile::tempdir().unwrap();
    let serving = initialize(directory.path());
    let foreign = initialize(foreign_directory.path());
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        cancellation.token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    publish_anchor(&serving, 0, vec![41; 150_001]);
    let (membership, _) = verified_read(&serving, &request);
    let protection = serving.read_protection_observer();
    let before = protection.snapshot().live_acquisitions();
    let mut task = membership.into_retirement();
    assert!(matches!(
        task.advance(&foreign.anchored_records().unwrap(), &request),
        Err(AnchoredRecordReadRetirementDenial::ForeignServing)
    ));
    assert_eq!(protection.snapshot().live_acquisitions(), before);
    let records = serving.anchored_records().unwrap();
    // First move the actual unique Arc payload, then release its frame. The
    // next stopped step must not repeat either accepted ownership transition.
    assert_eq!(
        task.advance(&records, &request).unwrap(),
        AnchoredRecordReadRetirementProgress::Advanced
    );
    assert_eq!(
        task.advance(&records, &request).unwrap(),
        AnchoredRecordReadRetirementProgress::Advanced
    );
    cancellation.cancel();
    assert!(matches!(
        task.advance(&records, &request),
        Err(AnchoredRecordReadRetirementDenial::Owner(
            PhysicalRecordReadRetirementDenial::Execution(_)
        ))
    ));
    assert_eq!(protection.snapshot().live_acquisitions(), before);
    let task = task
        .into_retired()
        .err()
        .expect("stop retains the same unfinished task");
    // The original stopped request remains stopped; generic fallback is
    // explicitly unbudgeted, with no replacement request or fabricated resume.
    drop(task);
    assert_eq!(protection.snapshot().live_acquisitions(), 0);
    // This borrowed facade has no custody; its last use ends the loan.
    assert_clean_close(serving);
    assert_clean_close(foreign);
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
}
