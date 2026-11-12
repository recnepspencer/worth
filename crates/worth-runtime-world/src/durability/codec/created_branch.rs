//! A selected branch retains only the effects its actual creation requested.
use super::{
    creation,
    fields::{number, Emit, Read},
    initial_components, owner_evidence,
};
use crate::durability::artifact::{
    WorldBranchCreatedRecord as Created, WorldCreatedBranchOwnerEvidence as Owners,
    WorldRelationalForkEvidence as Fork,
};
use crate::durability::{WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Medium};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::HomeIncarnation;
use worth_relational::facade::branch::RelationalBranchTransitionSequence;

pub(super) fn write(out: &mut Emit<'_>, created: &Created) -> Result<(), Frame> {
    created.validate_description()?;
    creation::write(out, &created.intent)?;
    initial_components::write(out, &created.components)?;
    match &created.owners {
        Owners::Reused => out(&[0])?,
        Owners::Relational(fork) => {
            out(&[1])?;
            fork_bytes(out, fork)?;
        }
        Owners::Signal(signal) => {
            out(&[2])?;
            owner_evidence::signal_bytes(out, signal)?;
        }
        Owners::RelationalAndSignal { relational, signal } => {
            out(&[3])?;
            fork_bytes(out, relational)?;
            owner_evidence::signal_bytes(out, signal)?;
        }
    }
    number(out, created.publishing_incarnation.get())
}

pub(super) fn read(
    input: &mut Read<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Created, Medium> {
    let intent = creation::read(input)?;
    let components = initial_components::read(input, request)?;
    let owners = match input.fixed::<1>()? {
        [0] => Owners::Reused,
        [1] => Owners::Relational(read_fork(input)?),
        [2] => Owners::Signal(owner_evidence::read_signal(input, request)?),
        [3] => Owners::RelationalAndSignal {
            relational: read_fork(input)?,
            signal: owner_evidence::read_signal(input, request)?,
        },
        _ => return Err(Frame::Malformed.into()),
    };
    let publishing_incarnation =
        HomeIncarnation::from_recorded(input.number()?).ok_or(Frame::Malformed)?;
    let created = Created {
        intent,
        components,
        owners,
        publishing_incarnation,
    };
    created.validate_description()?;
    Ok(created)
}

fn fork_bytes(out: &mut Emit<'_>, fork: &Fork) -> Result<(), Frame> {
    number(out, fork.sequence.0)?;
    out(&fork.digest)
}

fn read_fork(input: &mut Read<'_>) -> Result<Fork, Frame> {
    let sequence = RelationalBranchTransitionSequence(input.number()?);
    if sequence.0 == 0 {
        return Err(Frame::Malformed);
    }
    Ok(Fork {
        sequence,
        digest: input.fixed()?,
    })
}
