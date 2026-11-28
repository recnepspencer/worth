//! One little-endian envelope, with SHA-256 covering every payload byte.
use crate::{
    ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial,
    ExecutionStop, WorkCeilingDenial,
};
use worth_foundational::Sha256Digest;

pub const FRAMED_ENVELOPE_HEADER_BYTES: usize = 52;
const HEADER: usize = FRAMED_ENVELOPE_HEADER_BYTES;
const CHUNK: usize = 64 << 10;

#[derive(Debug, Clone, Copy)]
pub struct FramedEnvelopeSpec {
    pub magic: [u8; 8],
    pub supported_versions: &'static [u32],
    pub max_payload_bytes: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FramedEnvelopeError {
    TruncatedHeader,
    MagicMismatch { expected: [u8; 8], found: [u8; 8] },
    UnsupportedVersion { found: u32 },
    PayloadLengthMismatch { declared: u64, actual: u64 },
    PayloadTooLarge { declared: u64, limit: u64 },
    ChecksumMismatch,
    Execution(ExecutionStop),
    Allocation,
}

impl FramedEnvelopeSpec {
    fn validate_version(self, version: u32) -> Result<(), FramedEnvelopeError> {
        if !self.supported_versions.contains(&version) {
            return Err(FramedEnvelopeError::UnsupportedVersion { found: version });
        }
        Ok(())
    }

    fn validate_size(self, size: u64) -> Result<(), FramedEnvelopeError> {
        if size > self.max_payload_bytes {
            return Err(FramedEnvelopeError::PayloadTooLarge {
                declared: size,
                limit: self.max_payload_bytes,
            });
        }
        Ok(())
    }
}

pub fn charge_framed_bytes(
    request: &ExecutionRequest<'_, '_>,
    bytes: usize,
) -> Result<(), ExecutionStop> {
    request
        .run_native_with_checkpoint(|context| meter_chunk(context, bytes))?
        .0
}

fn meter_chunk(
    context: &mut crate::MapKernelContext<'_, '_>,
    bytes: usize,
) -> Result<(), ExecutionStop> {
    if context.has_accounting() {
        context.checkpoint(bytes as u64)
    } else {
        context.poll()
    }
    .map_err(WorkCeilingDenial::Stopped)
}

fn metered<R, E: From<FramedEnvelopeError>>(
    request: &ExecutionRequest<'_, '_>,
    operation: impl FnOnce(&mut dyn FnMut(usize) -> Result<(), FramedEnvelopeError>) -> Result<R, E>,
) -> Result<R, E> {
    request
        .run_native_with_checkpoint(|context| {
            operation(&mut |bytes| {
                meter_chunk(context, bytes).map_err(FramedEnvelopeError::Execution)
            })
        })
        .map_err(FramedEnvelopeError::Execution)?
        .0
}

pub fn decode_framed_envelope<'a>(
    spec: FramedEnvelopeSpec,
    bytes: &'a [u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(u32, &'a [u8]), FramedEnvelopeError> {
    let (version, digest, payload) = split_frame(spec, bytes)?;
    metered(request, |charge| {
        charge(HEADER)?;
        verify_digest(digest, payload, charge)
    })?;
    Ok((version, payload))
}

/// Share validation with callers whose inspected bytes have already been
/// charged at an enclosing boundary. The callback runs before every hash chunk.
pub fn inspect_framed_envelope<'a, E: From<FramedEnvelopeError>>(
    spec: FramedEnvelopeSpec,
    bytes: &'a [u8],
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<(u32, &'a [u8]), E> {
    let (version, digest, payload) = split_frame(spec, bytes)?;
    verify_digest(digest, payload, &mut charge)?;
    Ok((version, payload))
}

fn split_frame(
    spec: FramedEnvelopeSpec,
    bytes: &[u8],
) -> Result<(u32, &[u8], &[u8]), FramedEnvelopeError> {
    use FramedEnvelopeError as Error;
    let prefix = bytes.get(..12).ok_or(Error::TruncatedHeader)?;
    let found = prefix[..8].try_into().unwrap();
    if found != spec.magic {
        return Err(Error::MagicMismatch {
            expected: spec.magic,
            found,
        }
        .into());
    }
    let version = u32::from_le_bytes(prefix[8..12].try_into().unwrap());
    spec.validate_version(version)?;
    let (header, payload) = bytes
        .split_at_checked(HEADER)
        .ok_or(Error::TruncatedHeader)?;
    let declared = u64::from_le_bytes(header[12..20].try_into().unwrap());
    let actual = payload.len() as u64;
    spec.validate_size(actual)?;
    if declared != actual {
        return Err(Error::PayloadLengthMismatch { declared, actual }.into());
    }
    Ok((version, &header[20..], payload))
}

fn verify_digest<E: From<FramedEnvelopeError>>(
    expected: &[u8],
    payload: &[u8],
    charge: &mut dyn FnMut(usize) -> Result<(), E>,
) -> Result<(), E> {
    let mut digest = Sha256Digest::default();
    if payload.is_empty() {
        charge(0)?;
    }
    for chunk in payload.chunks(CHUNK) {
        charge(chunk.len())?;
        digest.update(chunk);
    }
    if digest.finish() != expected {
        return Err(FramedEnvelopeError::ChecksumMismatch.into());
    }
    Ok(())
}

type Sink<'a, E> = dyn FnMut(&[u8]) -> Result<(), E> + 'a;

fn prepare<E: From<FramedEnvelopeError>>(
    spec: FramedEnvelopeSpec,
    version: u32,
    charge: &mut dyn FnMut(usize) -> Result<(), FramedEnvelopeError>,
    emit: &mut impl FnMut(&mut Sink<'_, E>) -> Result<(), E>,
) -> Result<([u8; HEADER], usize), E> {
    let mut length = 0usize;
    let mut digest = Sha256Digest::default();
    emit(&mut |bytes| {
        length = length
            .checked_add(bytes.len())
            .ok_or(FramedEnvelopeError::Allocation)?;
        spec.validate_size(length as u64)?;
        for chunk in bytes.chunks(CHUNK) {
            charge(chunk.len())?;
            digest.update(chunk);
        }
        Ok(())
    })?;
    let mut header = [0; HEADER];
    header[..8].copy_from_slice(&spec.magic);
    header[8..12].copy_from_slice(&version.to_le_bytes());
    header[12..20].copy_from_slice(&(length as u64).to_le_bytes());
    header[20..].copy_from_slice(&digest.finish());
    let total = HEADER
        .checked_add(length)
        .ok_or(FramedEnvelopeError::Allocation)?;
    Ok((header, total))
}

fn write_payload<E: From<FramedEnvelopeError>>(
    charge: &mut dyn FnMut(usize) -> Result<(), FramedEnvelopeError>,
    length: usize,
    emit: &mut impl FnMut(&mut Sink<'_, E>) -> Result<(), E>,
    sink: &mut Sink<'_, E>,
) -> Result<(), E> {
    let mut actual = 0usize;
    emit(&mut |bytes| {
        actual = actual
            .checked_add(bytes.len())
            .ok_or(FramedEnvelopeError::Allocation)?;
        if actual > length {
            return Err(length_mismatch(length, actual).into());
        }
        for chunk in bytes.chunks(CHUNK) {
            charge(chunk.len())?;
            sink(chunk)?;
        }
        Ok(())
    })?;
    if actual != length {
        return Err(length_mismatch(length, actual).into());
    }
    Ok(())
}

fn length_mismatch(declared: usize, actual: usize) -> FramedEnvelopeError {
    FramedEnvelopeError::PayloadLengthMismatch {
        declared: declared as u64,
        actual: actual as u64,
    }
}

/// Encode a borrowed payload, charging the allocation while constructing it.
/// The returned Vec belongs to the caller. Durable owners use the funded array
/// entry point to carry the storage reservation through publication.
pub fn encode_framed_envelope(
    spec: FramedEnvelopeSpec,
    version: u32,
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<Vec<u8>, FramedEnvelopeError> {
    spec.validate_version(version)?;
    spec.validate_size(payload.len() as u64)?;
    metered(request, |charge| {
        let mut emit = |sink: &mut Sink<'_, FramedEnvelopeError>| sink(payload);
        let (header, total) = prepare(spec, version, charge, &mut emit)?;
        let _funding = request
            .reserve_retained_memory(total as u64)
            .map_err(FramedEnvelopeError::Execution)?;
        let mut frame = Vec::new();
        frame
            .try_reserve_exact(total)
            .map_err(|_| FramedEnvelopeError::Allocation)?;
        frame.extend_from_slice(&header);
        write_payload(charge, total - HEADER, &mut emit, &mut |chunk| {
            frame.extend_from_slice(chunk);
            Ok(())
        })?;
        Ok(frame)
    })
}

/// Author a funded envelope with a repeatable payload emitter.
pub fn encode_framed_array<E: From<FramedEnvelopeError>>(
    spec: FramedEnvelopeSpec,
    version: u32,
    request: &ExecutionRequest<'_, '_>,
    mut emit: impl FnMut(&mut Sink<'_, E>) -> Result<(), E>,
    backing: impl Fn(ExecutionRequestArrayDenial) -> E,
) -> Result<ExecutionArray<u8>, E> {
    build_framed_array(spec, version, request, &mut emit, &backing)?
        .seal()
        .map_err(backing)
}

/// Author shared immutable bytes, funding their control allocation as well.
pub fn encode_framed_bytes<E: From<FramedEnvelopeError>>(
    spec: FramedEnvelopeSpec,
    version: u32,
    request: &ExecutionRequest<'_, '_>,
    mut emit: impl FnMut(&mut Sink<'_, E>) -> Result<(), E>,
    backing: impl Fn(ExecutionRequestArrayDenial) -> E,
) -> Result<crate::ExecutionImmutableBytes, E> {
    build_framed_array(spec, version, request, &mut emit, &backing)?
        .seal_immutable_bytes()
        .map_err(backing)
}

fn build_framed_array<'r, 'a, E: From<FramedEnvelopeError>>(
    spec: FramedEnvelopeSpec,
    version: u32,
    request: &ExecutionRequest<'r, 'a>,
    emit: &mut impl FnMut(&mut Sink<'_, E>) -> Result<(), E>,
    backing: &impl Fn(ExecutionRequestArrayDenial) -> E,
) -> Result<ExecutionRequestArrayBuilder<'r, 'a, u8>, E> {
    spec.validate_version(version)?;
    metered(request, |charge| {
        let (header, total) = prepare(spec, version, charge, emit)?;
        let mut out = ExecutionRequestArrayBuilder::allocate(*request, total).map_err(backing)?;
        out.extend_from_slice(&header).map_err(backing)?;
        write_payload(charge, total - HEADER, emit, &mut |chunk| {
            out.extend_from_slice(chunk).map_err(backing)
        })?;
        Ok(out)
    })
}

/// Write to an enclosing owner's admitted sink without a second allocation.
/// The emitter must produce the same payload on both passes.
pub fn write_framed_envelope<E: From<FramedEnvelopeError>>(
    spec: FramedEnvelopeSpec,
    version: u32,
    request: &ExecutionRequest<'_, '_>,
    mut emit: impl FnMut(&mut Sink<'_, E>) -> Result<(), E>,
    sink: &mut Sink<'_, E>,
) -> Result<(), E> {
    spec.validate_version(version)?;
    metered(request, |charge| {
        let (header, total) = prepare(spec, version, charge, &mut emit)?;
        sink(&header)?;
        write_payload(charge, total - HEADER, &mut emit, sink)
    })
}

/// Fill the original buffer after an effect. The owner reserved the maximum
/// backing before the effect, so no new request can stop receipt construction.
pub fn fill_admitted_framed_buffer<E: From<FramedEnvelopeError>>(
    spec: FramedEnvelopeSpec,
    version: u32,
    out: &mut crate::ExecutionByteBuffer,
    mut emit: impl FnMut(&mut Sink<'_, E>) -> Result<(), E>,
    backing: impl Fn(crate::ExecutionAllocationDenial) -> E,
) -> Result<(), E> {
    spec.validate_version(version)?;
    let mut admitted = |_| Ok(());
    let (header, total) = prepare(spec, version, &mut admitted, &mut emit)?;
    let mut cursor = 0usize;
    let mut sink = |bytes: &[u8]| -> Result<(), E> {
        let written = out.len().saturating_sub(cursor).min(bytes.len());
        if written != 0 {
            out.overwrite(cursor, &bytes[..written]).map_err(&backing)?;
        }
        out.extend_from_slice(&bytes[written..]).map_err(&backing)?;
        cursor += bytes.len();
        Ok(())
    };
    sink(&header)?;
    write_payload(&mut admitted, total - HEADER, &mut emit, &mut sink)
}
