//! Explicit image graph rows must agree with every derived historical edge.
use super::*;
use crate::durability::{codec::fields::Read, WorldReadmissionDenial as Denial};
use worth_execution::ExecutionRequest;

pub(super) fn validate(
    read: &mut Read<'_>,
    image: &WorldCheckpointImage,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let count = super::decoding::count(read, 58)?;
    if count != image.history.len() {
        return Err(Denial::BranchLineage);
    }
    for (commit, expected) in image.history.iter() {
        super::framing::charge(request, 1).map_err(Denial::Medium)?;
        if read.number().map_err(frame)? != commit.ordinal() {
            return Err(Denial::BranchLineage);
        }
        head(read, expected)?;
    }
    let count = super::decoding::count(read, 76)?;
    if count != image.heads.len() {
        return Err(Denial::BranchLineage);
    }
    for (branch, expected) in image.heads.iter() {
        super::framing::charge(request, 1).map_err(Denial::Medium)?;
        read.matches_branch(branch.branch()).map_err(frame)?;
        head(read, expected)?;
    }
    Ok(())
}
fn frame(cause: crate::durability::WorldDurableFrameDenial) -> Denial {
    Denial::Medium(crate::durability::WorldDurableMediumDenial::Frame(cause))
}
fn head(read: &mut Read<'_>, expected: &WorldImageHead) -> Result<(), Denial> {
    let (branch, incarnation, commit, generation) = expected.fields();
    read.matches_branch(branch).map_err(frame)?;
    if read.number().map_err(frame)? != incarnation.ordinal()
        || read.number().map_err(frame)? != generation.get()
        || read.number().map_err(frame)? != commit.ordinal()
    {
        return Err(Denial::BranchLineage);
    }
    Ok(())
}
