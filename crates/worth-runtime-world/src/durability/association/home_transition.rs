//! The original World grammar is encoded once, then enclosed as Home-step DATA.
use super::*;
use worth_execution::ExecutionRequestArrayBuilder;
use worth_foundational::facade::{
    HomeProgramTransitionCaller, HomeProgramTransitionCallerDenial,
    HomeProgramTransitionPlanDescription, HomeProgramTransitionStepAssociation,
};

pub(crate) fn encode_home_transition_association(
    request: &ExecutionRequest<'_, '_>,
    plan: &HomeProgramTransitionPlanDescription,
    step: HomeProgramTransitionStepAssociation,
    original: &CompositeAttemptAssociation,
) -> Result<ExecutionArray<u8>, WorldDurableMediumDenial> {
    let bytes = encode_association(request, original)?;
    let description = HomeProgramTransitionCaller::for_step(plan, step, original, bytes.elements())
        .map_err(|cause| WorldDurableMediumDenial::Frame(frame_denial(cause)))?;
    let header = description.header();
    let length = header
        .len()
        .checked_add(bytes.elements().len())
        .filter(|length| {
            *length <= worth_relational::facade::mvcc::RelationalCallerAssociation::MAX_LEN
        })
        .ok_or(WorldDurableMediumDenial::Frame(
            WorldDurableFrameDenial::SizeLimit,
        ))?;
    let mut out = ExecutionRequestArrayBuilder::allocate(*request, length)
        .map_err(WorldDurableMediumDenial::Backing)?;
    for byte in header.iter().chain(bytes.elements()) {
        out.push(*byte).map_err(WorldDurableMediumDenial::Backing)?;
    }
    out.seal().map_err(WorldDurableMediumDenial::Backing)
}

pub(super) fn frame_denial(cause: HomeProgramTransitionCallerDenial) -> WorldDurableFrameDenial {
    match cause {
        HomeProgramTransitionCallerDenial::Malformed => WorldDurableFrameDenial::Malformed,
        HomeProgramTransitionCallerDenial::UnsupportedVersion(cause) => {
            WorldDurableFrameDenial::UnsupportedVersion(cause)
        }
        HomeProgramTransitionCallerDenial::Description(_) => {
            WorldDurableFrameDenial::IdentityMismatch
        }
        HomeProgramTransitionCallerDenial::Length => WorldDurableFrameDenial::SizeLimit,
    }
}
