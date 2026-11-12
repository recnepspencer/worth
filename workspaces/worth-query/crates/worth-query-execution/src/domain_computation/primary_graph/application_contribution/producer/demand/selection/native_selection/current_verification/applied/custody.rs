//! Checkpoint descriptions join the already admitted current Native holder.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture as Posture;
use worth_relational::facade::mvcc::{
    RecordedRequiredOutputRole, RequiredOutputPublicationMeaning, RequiredOutputRolePosture,
};
pub(super) fn matches(
    layout: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    candidate: &WorthQueryRetainedOutputCandidate,
    prior: &worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
    family: &'static str,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    if let Some(origin) = &candidate.required_publication {
        return Ok(origin.same_origin(prior));
    }
    let Some(checkpoint) = &candidate.recovered_checkpoint else {
        return Ok(false);
    };
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(prior.meaning().source_scope);
    if checkpoint.scope != scope {
        return Ok(false);
    }
    let Some(dependency) = checkpoint.producer_dependency else {
        return Ok(false);
    };
    let interrupted = |stop: worth_execution::WorkCeilingDenial| {
        candidate_support_denied(AttemptDenial::relational_read_stopped(stop.into(), family))
    };
    let missing = execution
        .run_native_with_checkpoint(
            |context| -> Result<Option<&str>, worth_execution::MapKernelStop> {
                for role in checkpoint.roles.iter() {
                    crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                        context,
                        role.entity_name.len().saturating_add(1) as u64,
                    )?;
                    if layout.entity_kind(&role.entity_name).is_none() {
                        return Ok(Some(role.entity_name.as_ref()));
                    }
                }
                Ok(None)
            },
        )
        .map_err(interrupted)?
        .0
        .map_err(|stop| interrupted(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
    if let Some(entity) = missing {
        return Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            format!("{family}: {entity}"),
        ));
    }
    let meaning = RequiredOutputPublicationMeaning {
        implementation: checkpoint.implementation,
        family: family.into(),
        source_scope: prior.meaning().source_scope,
        source_partition: worth_foundational::facade::CanonicalDigestId::new(
            checkpoint.source_partition,
        ),
        source_epoch: worth_foundational::facade::CanonicalDigestId::new(checkpoint.source),
    };
    prior
        .matches_recorded_description(
            execution,
            &checkpoint.producer,
            &dependency,
            &meaning,
            checkpoint
                .roles
                .iter()
                .map(|role| RecordedRequiredOutputRole {
                    role: &role.role,
                    entity_name: &role.entity_name,
                    kind: layout
                        .entity_kind(&role.entity_name)
                        .expect("validated immutable checkpoint role entity kind"),
                    posture: match role.posture {
                        Posture::Create => RequiredOutputRolePosture::Create,
                        Posture::Preserve => RequiredOutputRolePosture::Preserve,
                        Posture::Retire => RequiredOutputRolePosture::Retire,
                    },
                    entity: Some(role.entity),
                }),
        )
        .map_err(super::super::super::super::native_restoration::native_denial)
}
