//! Admit native retirement origins against the sealed installed producer owner.
use super::{DeclaredProducerBinding, WorthQueryInstalledApplicationProducerRegistry};
use crate::domain_computation::primary_graph::{
    application_contribution::WorthQueryProducerLifecyclePosture,
    schema_layout::WorthQueryPrimaryGraphLayout, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};
use worth_execution::{
    ExecutionRequest, ExecutionWorkCeiling, MapKernelContext, WorkCeilingDenial,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    durability::RecoveredRelationalRuntimeAuthority, identity::KindId,
    transactions::RelationalExecutionDenialCause,
};

impl<Schema: ApplicationSchema> WorthQueryInstalledApplicationProducerRegistry<Schema> {
    /// Recovery-lane work is bounded by reached native origins, each producer
    /// lookup, and that producer's applicability and output-role inventories.
    pub(in crate::domain_computation::primary_graph) fn validate_codec_transition_output_origins(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        recovered: &RecoveredRelationalRuntimeAuthority,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        execution.consult().map_err(request_denial)?;
        let result = execution.in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    charge(context, 1)?;
                    for (_, kind, producer, _) in recovered.codec_transition_output_origins() {
                        let entries = self.entries.len();
                        let levels = usize::BITS as usize - entries.max(1).leading_zeros() as usize;
                        let lookup = entries
                            .min(11)
                            .checked_mul(levels)
                            .and_then(|work| work.checked_add(1))
                            .and_then(|work| work.checked_mul(producer.len().checked_add(1)?))
                            .ok_or_else(work_overflow)?;
                        charge(context, lookup)?;
                        let entry = self.entries.get(producer).ok_or_else(|| {
                            rejection(producer, "native output producer is not installed")
                        })?;
                        if !supports_initial(&entry.declaration, context)? {
                            return Err(rejection(
                                producer,
                                "native output producer has no supported Initial applicability",
                            ));
                        }
                        if !creates_native_kind(&entry.declaration, layout, kind, context)? {
                            return Err(rejection(
                                producer,
                                "native output kind has no installed producer Create role",
                            ));
                        }
                    }
                    Ok(())
                },
            )
        });
        // A request stop takes precedence over a simultaneous semantic refusal.
        execution.consult().map_err(request_denial)?;
        let (result, _) = result.map_err(request_denial)?.map_err(request_denial)?;
        result
    }
}

fn supports_initial(
    declaration: &DeclaredProducerBinding,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, WorthQueryPrimaryGraphInstallationDenial> {
    for applicability in &declaration.applicability {
        charge(context, 1)?;
        if applicability.lifecycle() != WorthQueryProducerLifecyclePosture::Initial {
            continue;
        }
        for supported in &declaration.supported {
            let work = applicability
                .profile_kind()
                .len()
                .checked_add(1)
                .ok_or_else(work_overflow)?;
            charge(context, work)?;
            if supported == applicability {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn creates_native_kind(
    declaration: &DeclaredProducerBinding,
    layout: &WorthQueryPrimaryGraphLayout,
    kind: KindId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, WorthQueryPrimaryGraphInstallationDenial> {
    for role in &declaration.output_role_descriptors {
        charge(context, 1)?;
        if role.posture() == ApplicationMutationOutputPosture::Create
            && matches_native_kind(layout, role.entity(), kind, context)?
        {
            return Ok(true);
        }
    }
    for family in &declaration.output_role_families {
        charge(context, 1)?;
        if family
            .postures()
            .allows(ApplicationMutationOutputPosture::Create)
            && matches_native_kind(layout, family.entity(), kind, context)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn matches_native_kind(
    layout: &WorthQueryPrimaryGraphLayout,
    entity: &str,
    kind: KindId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, WorthQueryPrimaryGraphInstallationDenial> {
    let work = layout
        .entity_kind_lookup_work(entity)
        .ok_or_else(work_overflow)?;
    context
        .checkpoint_cost(work, work)
        .map_err(|stop| request_denial(WorkCeilingDenial::Stopped(stop)))?;
    Ok(layout.entity_kind(entity) == Some(kind))
}

fn charge(
    context: &mut MapKernelContext<'_, '_>,
    work: usize,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let work = u64::try_from(work).map_err(|_| work_overflow())?;
    context
        .checkpoint_cost(work, work)
        .map_err(|stop| request_denial(WorkCeilingDenial::Stopped(stop)))
}

fn work_overflow() -> WorthQueryPrimaryGraphInstallationDenial {
    execution_denial(RelationalExecutionDenialCause::WorkCounterOverflow)
}

fn request_denial(stop: WorkCeilingDenial) -> WorthQueryPrimaryGraphInstallationDenial {
    execution_denial(stop.into())
}

fn execution_denial(
    cause: RelationalExecutionDenialCause,
) -> WorthQueryPrimaryGraphInstallationDenial {
    let translated = crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause, None);
    let kind = match translated {
        Ok(kind) => DenialKind::ExecutionDenied { kind },
        Err(kind) => DenialKind::ExecutionControlStopped { kind },
    };
    WorthQueryPrimaryGraphInstallationDenial::new(
        kind,
        "native codec output-origin validation request stopped",
    )
}

fn rejection(producer: &str, reason: &str) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        DenialKind::CheckpointRecoveryRejected,
        format!("{reason}: {producer}"),
    )
}
