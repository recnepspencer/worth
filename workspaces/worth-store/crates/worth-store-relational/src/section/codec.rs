//! The unchanged WSRCKPT1 version-3 native section envelope.
use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::durability::{
    RelationalDurableCheckpointSource, RelationalNativeCheckpoint,
};
use worth_relational::facade::publication::PatchStreamPosition;

const MAGIC: &[u8; 8] = b"WSRCKPT1";
const VERSION: u16 = 3;
const HEADER_BYTES: usize = 26;

pub(super) fn decode(
    bytes: &[u8],
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<RelationalDurableCheckpointSource, super::RelationalSectionReadmissionDenial> {
    use super::RelationalSectionReadmissionDenial as Denial;
    let (covered, transitions) = coverage(bytes).ok_or(Denial::MalformedSection)?;
    let source = &bytes[HEADER_BYTES..];
    let mut admitted =
        worth_execution::ExecutionRequestArrayBuilder::<u8>::allocate(*execution, source.len())
            .map_err(Denial::Payload)?;
    let (copied, _) = execution
        .run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(source.len() as u64)?;
            } else {
                context.poll()?;
            }
            Ok::<_, worth_execution::MapKernelStop>(admitted.extend_from_slice(source))
        })
        .map_err(Denial::Execution)?;
    copied
        .map_err(|cause| Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(cause)))?
        .map_err(Denial::Payload)?;
    let backing = admitted
        .seal()
        .map_err(Denial::Payload)?
        .into_immutable_bytes();
    let checkpoint =
        RelationalNativeCheckpoint::from_untrusted_bytes_region(backing, 0..source.len())
            .map_err(|_| Denial::MalformedSection)?;
    Ok(RelationalDurableCheckpointSource::new(
        PatchStreamPosition(covered),
        RelationalBranchTransitionSequence(transitions),
        checkpoint,
    ))
}

pub(super) fn coverage(bytes: &[u8]) -> Option<(u64, u64)> {
    let rest = bytes.strip_prefix(MAGIC)?;
    let (version, rest) = rest.split_first_chunk::<2>()?;
    if u16::from_le_bytes(*version) != VERSION {
        return None;
    }
    let (position, rest) = rest.split_first_chunk::<8>()?;
    let (transitions, _) = rest.split_first_chunk::<8>()?;
    Some((
        u64::from_le_bytes(*position),
        u64::from_le_bytes(*transitions),
    ))
}

pub(super) fn encode(checkpoint: &RelationalNativeCheckpoint) -> Option<Vec<u8>> {
    let covered = checkpoint.covered_position()?;
    let transitions = checkpoint.covered_branch_transitions()?;
    let mut bytes = Vec::with_capacity(HEADER_BYTES.checked_add(checkpoint.bytes().len())?);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&covered.0.to_le_bytes());
    bytes.extend_from_slice(&transitions.0.to_le_bytes());
    bytes.extend_from_slice(checkpoint.bytes());
    Some(bytes)
}
