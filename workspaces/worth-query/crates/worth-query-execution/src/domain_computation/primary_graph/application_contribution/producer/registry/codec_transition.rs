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
                    for ((_, kind, producer, _), (_, _, birth, _, role_posture, publication_posture)) in recovered
                        .codec_transition_output_origins()
                        .zip(recovered.codec_transition_output_births())
                    {
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
                        let birth_entry = self.entries.get(birth).ok_or_else(|| {
                            rejection(birth, "native birth producer is not installed")
                        })?;
                        if !supports_lifecycle(
                            &birth_entry.declaration,
                            WorthQueryProducerLifecyclePosture::Initial,
                            context,
                        )? && !supports_lifecycle(
                            &birth_entry.declaration,
                            WorthQueryProducerLifecyclePosture::Preserve,
                            context,
                        )? || !outputs_native_kind(
                            &birth_entry.declaration,
                            layout,
                            kind,
                            ApplicationMutationOutputPosture::Create,
                            context,
                        )? {
                            return Err(rejection(
                                birth,
                                "native output lacks installed actual Create birth meaning",
                            ));
                        }
                        let (lifecycle, _output_posture) = match publication_posture {
                            worth_relational::facade::mvcc::RequiredOutputRolePosture::Create => (
                                WorthQueryProducerLifecyclePosture::Initial,
                                ApplicationMutationOutputPosture::Create,
                            ),
                            worth_relational::facade::mvcc::RequiredOutputRolePosture::Preserve => {
                                (
                                    WorthQueryProducerLifecyclePosture::Preserve,
                                    ApplicationMutationOutputPosture::Preserve,
                                )
                            }
                            worth_relational::facade::mvcc::RequiredOutputRolePosture::Retire => {
                                return Err(rejection(
                                    producer,
                                    "retirement has no present recoverable payload",
                                ))
                            }
                        };
                        if !supports_lifecycle(&entry.declaration, lifecycle, context)?
                            || !outputs_native_kind(
                                &entry.declaration,
                                layout,
                                kind,
                                match role_posture {
                                    worth_relational::facade::mvcc::RequiredOutputRolePosture::Create => ApplicationMutationOutputPosture::Create,
                                    worth_relational::facade::mvcc::RequiredOutputRolePosture::Preserve => ApplicationMutationOutputPosture::Preserve,
                                    worth_relational::facade::mvcc::RequiredOutputRolePosture::Retire => ApplicationMutationOutputPosture::Retire,
                                },
                                context,
                            )?
                        {
                            return Err(rejection(
                                producer,
                                "latest native publication differs from installed producer meaning",
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

fn supports_lifecycle(
    declaration: &DeclaredProducerBinding,
    lifecycle: WorthQueryProducerLifecyclePosture,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, WorthQueryPrimaryGraphInstallationDenial> {
    for applicability in &declaration.applicability {
        charge(context, 1)?;
        if applicability.lifecycle() != lifecycle {
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

fn outputs_native_kind(
    declaration: &DeclaredProducerBinding,
    layout: &WorthQueryPrimaryGraphLayout,
    kind: KindId,
    posture: ApplicationMutationOutputPosture,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, WorthQueryPrimaryGraphInstallationDenial> {
    for role in &declaration.output_role_descriptors {
        charge(context, 1)?;
        if role.posture() == posture && matches_native_kind(layout, role.entity(), kind, context)? {
            return Ok(true);
        }
    }
    for family in &declaration.output_role_families {
        charge(context, 1)?;
        if family.postures().allows(posture)
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
