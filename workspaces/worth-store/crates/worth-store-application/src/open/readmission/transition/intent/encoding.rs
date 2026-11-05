//! Encode the whole plan and original World assignment before any owner effect.
use super::*;
use crate::descriptor::binding;
use crate::open::readmission::transition::ReservedApplicationHomeTransitionPlan;
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequest;

pub(in crate::open::readmission::transition) fn encode(
    reserved: &ReservedApplicationHomeTransitionPlan,
    predecessor: &ApplicationHomeBinding,
    successor: &ApplicationHomeBinding,
    request: &ExecutionRequest<'_, '_>,
) -> Result<crate::frame_encoding::EncodedHomeFrame, ApplicationHomeTransitionIntentDenial> {
    use ApplicationHomeTransitionIntentDenial as Denial;
    let plan = reserved.description();
    let source = plan.predecessor();
    let native = reserved
        .native
        .to_json_in(request)
        .map_err(Denial::Native)?;
    let predecessor_size = binding::size(predecessor).ok_or(Denial::SizeLimit)?;
    let successor_size = binding::size(successor).ok_or(Denial::SizeLimit)?;
    let size = 192usize
        .checked_add(source.branch().name().len())
        .and_then(|size| size.checked_add(native.len()))
        .and_then(|size| size.checked_add(predecessor_size))
        .and_then(|size| size.checked_add(successor_size))
        .and_then(|size| {
            reserved
                .assignments()
                .len()
                .checked_mul(8)
                .and_then(|n| size.checked_add(n))
        })
        .filter(|size| *size <= MAXIMUM)
        .ok_or(Denial::SizeLimit)?;
    if reserved.assignments().len() != plan.steps() as usize {
        return Err(Denial::AssignmentCount);
    }
    let frame = crate::frame_encoding::encode(size, request, |out| {
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.get().to_le_bytes());
        out.extend_from_slice(&plan.identity().home().bytes());
        out.extend_from_slice(&plan.identity().intent_position().get().to_le_bytes());
        out.extend_from_slice(&source.branch().owner().slot().get().to_le_bytes());
        text(source.branch().name(), out);
        out.extend_from_slice(&source.incarnation().ordinal().to_le_bytes());
        out.extend_from_slice(&source.generation().get().to_le_bytes());
        out.extend_from_slice(&source.selected_commit().ordinal().to_le_bytes());
        out.extend_from_slice(&plan.digest().bytes());
        out.extend_from_slice(&plan.steps().to_le_bytes());
        out.extend_from_slice(&plan.target_meaning().step().to_le_bytes());
        out.extend_from_slice(&plan.target_meaning().catalog().bytes());
        out.extend_from_slice(
            &plan
                .target_meaning()
                .whole_candidate_validation_version()
                .to_le_bytes(),
        );
        bytes(native.as_ref(), out);
        out.extend_from_slice(&(predecessor_size as u32).to_le_bytes());
        binding::emit(predecessor, out);
        out.extend_from_slice(&(successor_size as u32).to_le_bytes());
        binding::emit(successor, out);
        for assignment in reserved.assignments() {
            out.extend_from_slice(&assignment.attempt().ordinal().to_le_bytes());
        }
        let checksum = Sha256::digest(out.as_slice());
        out.extend_from_slice(&checksum);
    })
    .map_err(Denial::Execution)?;
    debug_assert_eq!(frame.bytes.len(), size);
    Ok(frame)
}
fn bytes(value: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value);
}
fn text(value: &str, out: &mut Vec<u8>) {
    bytes(value.as_bytes(), out);
}
