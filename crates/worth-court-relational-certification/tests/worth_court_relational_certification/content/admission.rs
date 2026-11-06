use super::{fixture::*, original};
use std::{
    any::Any,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};
use worth_foundational::facade::{CanonicalDigestId, ContentFormat};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::content::*;

#[test]
fn content_above_frame_bound_refuses_at_admit_without_retained_effects() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let input = Grid {
        rows: 1,
        columns: 2_097_152,
        samples: vec![0; 2_097_152],
    };
    let before = request.memory().unwrap().available_memory_bytes();
    let denial =
        RelationalContentFrame::encode(&execution, &format("sample-grid", 1, false), &input)
            .unwrap_err();
    assert_eq!(
        denial,
        RelationalContentDenial::SizeLimit {
            observed: 16_777_259,
            maximum: 16_777_216
        }
    );
    assert_eq!(request.memory().unwrap().available_memory_bytes(), before);
}

#[test]
fn content_write_and_read_work_ceilings_refuse_without_custody_leaks() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let format = format("sample-grid", 1, false);
    let frame = RelationalContentFrame::encode(&execution, &format, &grid()).unwrap();
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(8192), Some(30)),
    );
    for write in [true, false] {
        let bounded = SerialRequest::from_policy(&policy, cancel.token(), None);
        let execution = ExecutionRequest::serial(&bounded);
        let result = if write {
            RelationalContentFrame::encode(&execution, &format, &grid())
        } else {
            RelationalContentFrame::readmit(&execution, &format, frame.identity(), frame.bytes())
        };
        assert!(matches!(result, Err(RelationalContentDenial::Execution(_))));
        assert_eq!(
            bounded.memory().unwrap().available_memory_bytes(),
            Some(8192)
        );
    }
}

#[derive(Debug)]
struct Alternating {
    codec: Codec,
    other: ContentFormat,
    calls: AtomicUsize,
}
impl RelationalContentCodec for Alternating {
    fn format(&self) -> &ContentFormat {
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            &self.codec.descriptor
        } else {
            &self.other
        }
    }
    fn payload_length(
        &self,
        input: &dyn Any,
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<usize, RelationalContentDenial> {
        self.codec.payload_length(input, control)
    }
    fn encode(
        &self,
        input: &dyn Any,
        sink: &mut RelationalContentSink<'_>,
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        self.codec.encode(input, sink, control)
    }
    fn validate(
        &self,
        payload: &[u8],
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        self.codec.validate(payload, control)
    }
    fn verify_encoding(
        &self,
        input: &dyn Any,
        payload: &[u8],
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        self.codec.verify_encoding(input, payload, control)
    }
}

#[test]
fn content_installed_descriptor_cannot_change_when_executable_codec_metadata_alternates() {
    let codec = Arc::new(Alternating {
        codec: Codec {
            descriptor: ContentFormat::new("sample-grid", 1).unwrap(),
            alter: false,
        },
        other: ContentFormat::new("other-format", 2).unwrap(),
        calls: AtomicUsize::new(0),
    });
    let mut registry = RelationalContentFormatRegistry::default();
    let id = CanonicalDigestId::new([1; 32]);
    registry.register(id, codec.clone()).unwrap();
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let format = registry.get(id).unwrap();
    let frame = RelationalContentFrame::encode(&execution, format, &grid()).unwrap();
    assert_eq!(codec.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        frame.format(),
        &ContentFormat::new("sample-grid", 1).unwrap()
    );
    RelationalContentFrame::readmit(&execution, format, frame.identity(), frame.bytes()).unwrap();
    assert_eq!(codec.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn content_each_readmitted_wrapper_retains_its_own_original_metadata_grant() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let format = format("sample-grid", 1, false);
    let frame = RelationalContentFrame::encode(&execution, &format, &grid()).unwrap();
    let record = frame.record();
    let before = request.memory().unwrap().available_memory_bytes().unwrap();
    let mut wrappers = Vec::new();
    let mut remaining = before;
    for _ in 0..128 {
        wrappers
            .push(RelationalContentFrame::readmit_record(&execution, &format, &record).unwrap());
        let after = request.memory().unwrap().available_memory_bytes().unwrap();
        assert!(
            after < remaining,
            "a retained descriptor allocation needs its own grant"
        );
        remaining = after;
    }
    drop(wrappers);
    assert_eq!(
        request.memory().unwrap().available_memory_bytes(),
        Some(before)
    );
}

#[test]
fn content_correctly_hashed_overflowing_dimensions_are_typed_noncanonical() {
    use sha2::{Digest, Sha256};
    use worth_foundational::facade::ContentRefId;
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let format = format("sample-grid", 1, false);
    let frame = RelationalContentFrame::encode(&execution, &format, &grid()).unwrap();
    let mut malformed = frame.bytes().to_vec();
    malformed[35..43].fill(255);
    let mut independent = Sha256::new();
    independent.update(b"worth.native.content.v1\0");
    independent.update(&malformed);
    let identity = ContentRefId(independent.finalize().into());
    assert_eq!(
        RelationalContentFrame::readmit(&execution, &format, identity, &malformed).unwrap_err(),
        RelationalContentDenial::NonCanonicalContent
    );
}

#[test]
fn content_concurrent_equal_frames_have_one_immutable_identity() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let format = format("sample-grid", 1, false);
    let frames = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..4)
            .map(|_| {
                let request = &request;
                let format = &format;
                scope.spawn(move || {
                    RelationalContentFrame::encode(
                        &ExecutionRequest::serial(request),
                        format,
                        &grid(),
                    )
                    .unwrap()
                })
            })
            .collect();
        jobs.into_iter()
            .map(|job| job.join().unwrap())
            .collect::<Vec<_>>()
    });
    for frame in &frames {
        assert_eq!(
            frame.identity().to_string(),
            "f4fb54508d898711c5770a58e4cff9989c1c302e7969d842891c0f5f02dc78e8"
        );
    }
    drop(frames);
    assert_eq!(
        request.memory().unwrap().available_memory_bytes(),
        Some(64 * 1024 * 1024)
    );
}
