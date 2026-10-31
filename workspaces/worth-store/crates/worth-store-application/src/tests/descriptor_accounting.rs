use crate::facade::{
    admit_home, recover_home, ApplicationHomeDescriptorDenial, ApplicationHomeOpenRefusal,
};
use worth_execution::{
    CancellationSource, ExecutionRequest, MapKernelStop, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn policy(work: u64) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), None, Some(work)),
    )
}

fn assert_no_descriptor_or_native_commit(path: &std::path::Path) {
    let opened = crate::open::physical_admission::open(
        path,
        crate::open::physical_admission::PhysicalOpening::production(),
    )
    .unwrap();
    assert!(opened
        .serving
        .anchored_records()
        .unwrap()
        .read(
            worth_store::physical_runtime::AnchoredRecordName::new("home").unwrap(),
            64 << 10
        )
        .unwrap()
        .is_none());
    assert!(opened
        .serving
        .canonical_commits()
        .unwrap()
        .head()
        .unwrap()
        .is_none());
    let _closed = opened.serving.close();
}

/// Quote from the producer's actual landed envelope, independently of the
/// encoder and its size function: the program binding frame plus descriptor.
fn quoted_work(bytes: &[u8]) -> u64 {
    assert_eq!(&bytes[..8], b"WSRHOME1");
    let mut at = 34; // magic, version, namespace identity, incarnation
    for _ in 0..2 {
        let length = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        at += 4 + length;
    }
    assert_eq!(u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap()), 2);
    at += 2;
    assert_eq!(u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap()), 1);
    let binding_bytes = u32::from_le_bytes(bytes[at + 2..at + 6].try_into().unwrap()) as u64;
    assert!(binding_bytes > 0);
    bytes.len() as u64 + binding_bytes
}

#[test]
fn descriptor_admission_bills_both_frames_to_one_unwrapped_original_request() {
    let directory = tempfile::tempdir().unwrap();
    let setup = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let setup_execution = ExecutionRequest::serial(&setup);
    let quoted = directory.path().join("quote");
    let recovered = recover_home(super::program::request(&quoted), &setup_execution).unwrap();
    let admitted = admit_home(recovered, &setup_execution).unwrap();
    let work = admitted.consume(|home| {
        let record = home
            .context
            .serving
            .anchored_records()
            .unwrap()
            .read(
                worth_store::physical_runtime::AnchoredRecordName::new("home").unwrap(),
                64 << 10,
            )
            .unwrap()
            .unwrap();
        quoted_work(record.bytes())
    });
    let short = directory.path().join("one-short");
    let recovered = recover_home(super::program::request(&short), &setup_execution).unwrap();
    let original =
        SerialRequest::from_policy(&policy(work - 1), CancellationSource::new().token(), None);
    // Deliberately no caller-created outer scope: production owns its entry.
    assert!(matches!(
        admit_home(recovered, &ExecutionRequest::serial(&original)),
        Err(ApplicationHomeOpenRefusal::Descriptor(
            ApplicationHomeDescriptorDenial::Execution(WorkCeilingDenial::Stopped(
                MapKernelStop::WorkCeiling
            ))
        ))
    ));
    assert_no_descriptor_or_native_commit(&short);
    let funded = directory.path().join("funded");
    let recovered = recover_home(super::program::request(&funded), &setup_execution).unwrap();
    let original =
        SerialRequest::from_policy(&policy(work), CancellationSource::new().token(), None);
    let admitted = admit_home(recovered, &ExecutionRequest::serial(&original)).unwrap();
    admitted.consume(|home| {
        let record = home
            .context
            .serving
            .anchored_records()
            .unwrap()
            .read(
                worth_store::physical_runtime::AnchoredRecordName::new("home").unwrap(),
                64 << 10,
            )
            .unwrap()
            .unwrap();
        assert_eq!(quoted_work(record.bytes()), work);
    });
}

#[test]
fn descriptor_admission_preserves_original_cancellation_before_any_descriptor_or_commit_effect() {
    let directory = tempfile::tempdir().unwrap();
    let setup = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let recovered = recover_home(
        super::program::request(directory.path()),
        &ExecutionRequest::serial(&setup),
    )
    .unwrap();
    let cancellation = CancellationSource::new();
    let original = SerialRequest::from_policy(&policy(u64::MAX), cancellation.token(), None);
    cancellation.cancel();
    assert!(matches!(
        admit_home(recovered, &ExecutionRequest::serial(&original)),
        Err(ApplicationHomeOpenRefusal::Descriptor(
            ApplicationHomeDescriptorDenial::Execution(WorkCeilingDenial::Stopped(
                MapKernelStop::Cancelled
            ))
        ))
    ));
    assert_no_descriptor_or_native_commit(directory.path());
}

#[test]
fn descriptor_admission_bills_both_frames_to_one_unwrapped_leased_request() {
    use worth_execution::{ExecutionResourceLease, ExecutionWorkCeiling, LeaseRequest};
    let directory = tempfile::tempdir().unwrap();
    let setup = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let setup_execution = ExecutionRequest::serial(&setup);
    let quote = directory.path().join("quote");
    let recovered = recover_home(super::program::request(&quote), &setup_execution).unwrap();
    // Observe the complete SDK bill as well as independently quoting the
    // landed bytes. The outer scope is for this diagnostic setup only.
    let (admitted, report) = setup_execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            admit_home(recovered, &setup_execution)
        })
        .unwrap();
    let work = admitted.unwrap().consume(|home| {
        let record = home
            .context
            .serving
            .anchored_records()
            .unwrap()
            .read(
                worth_store::physical_runtime::AnchoredRecordName::new("home").unwrap(),
                64 << 10,
            )
            .unwrap()
            .unwrap();
        quoted_work(record.bytes())
    });
    println!(
        "descriptor invoice: quoted={work}, actual={:?}",
        report.accounted_cost()
    );
    assert_eq!(
        report.accounted_cost(),
        Some((work, work)),
        "the quote is the exact complete SDK work/span bill, not an upper bound"
    );
    let short = directory.path().join("one-short");
    let recovered = recover_home(super::program::request(&short), &setup_execution).unwrap();
    let original = ExecutionResourceLease::serial(LeaseRequest {
        policy: policy(work - 1),
        cancellation: CancellationSource::new().token(),
        deadline: None,
    })
    .unwrap();
    // This is the caller's real lease, not a policy-created replacement inside
    // production. No active outer SDK scope conceals a fresh grant per frame.
    let outcome = admit_home(recovered, &ExecutionRequest::leased(&original));
    assert!(
        matches!(
            outcome,
            Err(ApplicationHomeOpenRefusal::Descriptor(
                ApplicationHomeDescriptorDenial::Execution(WorkCeilingDenial::Stopped(
                    MapKernelStop::WorkCeiling
                ))
            ))
        ),
        "the complete leased entry must refuse its one-unit-short grant: {:?}",
        outcome.map(|_| ())
    );
    assert_no_descriptor_or_native_commit(&short);

    let funded = directory.path().join("funded");
    let recovered = recover_home(super::program::request(&funded), &setup_execution).unwrap();
    let original = ExecutionResourceLease::serial(LeaseRequest {
        policy: policy(work),
        cancellation: CancellationSource::new().token(),
        deadline: None,
    })
    .unwrap();
    let admitted = admit_home(recovered, &ExecutionRequest::leased(&original)).unwrap();
    admitted.consume(|home| {
        let record = home
            .context
            .serving
            .anchored_records()
            .unwrap()
            .read(
                worth_store::physical_runtime::AnchoredRecordName::new("home").unwrap(),
                64 << 10,
            )
            .unwrap()
            .unwrap();
        assert_eq!(quoted_work(record.bytes()), work);
    });

    let cancelled = directory.path().join("cancelled");
    let recovered = recover_home(super::program::request(&cancelled), &setup_execution).unwrap();
    let cancellation = CancellationSource::new();
    let original = ExecutionResourceLease::serial(LeaseRequest {
        policy: policy(work),
        cancellation: cancellation.token(),
        deadline: None,
    })
    .unwrap();
    cancellation.cancel();
    assert!(matches!(
        admit_home(recovered, &ExecutionRequest::leased(&original)),
        Err(ApplicationHomeOpenRefusal::Descriptor(
            ApplicationHomeDescriptorDenial::Execution(WorkCeilingDenial::Stopped(
                MapKernelStop::Cancelled
            ))
        ))
    ));
    assert_no_descriptor_or_native_commit(&cancelled);
}
