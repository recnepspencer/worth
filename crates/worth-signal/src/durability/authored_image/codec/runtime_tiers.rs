//! Closed tier policy tags; caller tier keys are stable program names.
use super::{Denial, Read, Write};
use crate::data::checkpoint::CheckpointBarrier;
use crate::data::tier::{DependencyMode, DirtyPropagation, EvaluationTrigger};

pub(super) fn write(
    out: &mut Write<'_>,
    dependency: DependencyMode,
    propagation: DirtyPropagation,
    trigger: EvaluationTrigger,
) -> Result<(), Denial> {
    out.byte(match dependency {
        DependencyMode::Static => 0,
        DependencyMode::AutoDiscovered => 1,
    })?;
    out.byte(match propagation {
        DirtyPropagation::Batched => 0,
        DirtyPropagation::Immediate => 1,
    })?;
    match trigger {
        EvaluationTrigger::Checkpoint(barrier) => {
            out.byte(0)?;
            out.byte(match barrier {
                CheckpointBarrier::PerMutation => 0,
                CheckpointBarrier::PerOperation => 1,
                CheckpointBarrier::PerCommit => 2,
                CheckpointBarrier::OnDemandRead => 3,
            })
        }
        EvaluationTrigger::LazyPull => out.byte(1),
        EvaluationTrigger::OnDemand => out.byte(2),
        EvaluationTrigger::Async => out.byte(3),
    }
}
pub(super) fn read(
    input: &mut Read<'_>,
) -> Result<(DependencyMode, DirtyPropagation, EvaluationTrigger), Denial> {
    let dependency = match input.byte()? {
        0 => DependencyMode::Static,
        1 => DependencyMode::AutoDiscovered,
        _ => return Err(Denial::Malformed),
    };
    let propagation = match input.byte()? {
        0 => DirtyPropagation::Batched,
        1 => DirtyPropagation::Immediate,
        _ => return Err(Denial::Malformed),
    };
    let trigger = match input.byte()? {
        0 => EvaluationTrigger::Checkpoint(match input.byte()? {
            0 => CheckpointBarrier::PerMutation,
            1 => CheckpointBarrier::PerOperation,
            2 => CheckpointBarrier::PerCommit,
            3 => CheckpointBarrier::OnDemandRead,
            _ => return Err(Denial::Malformed),
        }),
        1 => EvaluationTrigger::LazyPull,
        2 => EvaluationTrigger::OnDemand,
        3 => EvaluationTrigger::Async,
        _ => return Err(Denial::Malformed),
    };
    Ok((dependency, propagation, trigger))
}
