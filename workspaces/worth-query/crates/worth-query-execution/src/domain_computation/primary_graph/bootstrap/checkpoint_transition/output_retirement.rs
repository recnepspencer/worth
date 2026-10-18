//! Current correspondence may retire only with its complete native output.
use super::denial;
use crate::domain_computation::primary_graph::{
    application_checkpoint::DecodedApplicationCheckpoint, WorthQueryPrimaryGraphInstallationDenial,
};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, MapKernelContext};
use worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority;

pub(super) fn retire(
    execution: &ExecutionRequest<'_, '_>,
    recovered: &RecoveredRelationalRuntimeAuthority,
    checkpoint: &mut DecodedApplicationCheckpoint,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    // This first supported cutover retires whole output records. A partly
    // classified correspondence cannot survive as a target-current claim.
    let result = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| validate(checkpoint, recovered, context),
        )
    });
    execution.consult().map_err(request)?;
    result.map_err(request)?.map_err(request)?.0?;
    // Only this candidate's reconstructed metadata changes. The original
    // checkpoint and its historical native roots retain their source meaning.
    checkpoint.accepted_outputs.clear();
    Ok(())
}

fn validate(
    checkpoint: &DecodedApplicationCheckpoint,
    recovered: &RecoveredRelationalRuntimeAuthority,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for output in &checkpoint.accepted_outputs {
        charge(context, 1)?;
        if output.roles.is_empty() {
            return Err(denial(
                "codec recovery cannot classify an empty output correspondence",
            ));
        }
        for role in output.roles.iter() {
            charge(context, 1)?;
            let mut matched = false;
            for (entity, _, producer, dependency) in recovered.codec_transition_output_origins() {
                let work = 1_u64
                    .saturating_add(producer.len() as u64)
                    .saturating_add(output.producer.len() as u64)
                    .saturating_add(32);
                charge(context, work)?;
                if entity == role.entity
                    && producer == output.producer.as_str()
                    && output.producer_dependency.as_ref() == Some(dependency)
                {
                    matched = true;
                    break;
                }
            }
            if !matched {
                return Err(denial("codec recovery requires complete performed native provenance for every accepted output role"));
            }
        }
    }
    Ok(())
}

fn charge(
    context: &mut MapKernelContext<'_, '_>,
    work: u64,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    context
        .checkpoint(work)
        .map_err(|stop| request(worth_execution::WorkCeilingDenial::Stopped(stop)))
}

fn request(cause: worth_execution::WorkCeilingDenial) -> WorthQueryPrimaryGraphInstallationDenial {
    super::super::request_read_denial::read(cause.into())
}
