use std::num::NonZeroUsize;
use worth_execution::{
    decode_framed_envelope, encode_framed_envelope, CancellationSource, ExecutionRequest,
    FramedEnvelopeError, FramedEnvelopeSpec, MapKernelStop, SerialRequest, WorkCeilingDenial,
    FRAMED_ENVELOPE_HEADER_BYTES as HEADER,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

const CHUNK: usize = 64 << 10;
const SPEC: FramedEnvelopeSpec = FramedEnvelopeSpec {
    magic: *b"TESTFRAM",
    supported_versions: &[7],
    max_payload_bytes: 2 * CHUNK as u64,
};

fn serial(work_ceiling: Option<u64>) -> SerialRequest {
    SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, work_ceiling),
        ),
        CancellationSource::new().token(),
        None,
    )
}

#[test]
fn envelope_matches_the_wire_layout_and_authenticates_the_payload() {
    let serial = serial(None);
    let request = ExecutionRequest::serial(&serial);
    let frame = encode_framed_envelope(SPEC, 7, b"abc", &request).unwrap();
    // SHA-256 known answer for abc.
    let digest = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];
    assert_eq!(&frame[..20], b"TESTFRAM\x07\0\0\0\x03\0\0\0\0\0\0\0");
    assert_eq!(&frame[20..52], &digest);
    assert_eq!(
        decode_framed_envelope(SPEC, &frame, &request),
        Ok((7, b"abc".as_slice()))
    );
    let mut corrupted = frame;
    corrupted[52] ^= 1;
    assert_eq!(
        decode_framed_envelope(SPEC, &corrupted, &request),
        Err(FramedEnvelopeError::ChecksumMismatch)
    );
}

#[test]
fn the_work_ceiling_applies_across_chunks_and_passes() {
    let payload = vec![9; CHUNK + 32];
    let unlimited = serial(None);
    let frame = encode_framed_envelope(SPEC, 7, &payload, &ExecutionRequest::serial(&unlimited))
        .expect("unrestricted request");
    let decode_budget = serial(Some((HEADER + CHUNK + 16) as u64));
    let request = ExecutionRequest::serial(&decode_budget);
    assert!(matches!(
        decode_framed_envelope(SPEC, &frame, &request),
        Err(FramedEnvelopeError::Execution(WorkCeilingDenial::Stopped(
            MapKernelStop::WorkCeiling
        )))
    ));
    let encode_budget = serial(Some((2 * CHUNK) as u64));
    let request = ExecutionRequest::serial(&encode_budget);
    assert!(matches!(
        encode_framed_envelope(SPEC, 7, &payload, &request),
        Err(FramedEnvelopeError::Execution(WorkCeilingDenial::Stopped(
            MapKernelStop::WorkCeiling
        )))
    ));
}
