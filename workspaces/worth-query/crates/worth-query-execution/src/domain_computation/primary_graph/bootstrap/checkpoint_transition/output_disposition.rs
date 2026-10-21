//! Native-founded retirement or historical retention of complete output rows.
use super::{denial, preserved_outputs::PreservedOutputs, publication::InstalledCheckpointTarget};
use crate::domain_computation::primary_graph::{
    application_checkpoint::{CheckpointMetadataFunding, DecodedApplicationCheckpoint},
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial as Denial,
};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, MapKernelContext};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis, durability::RecoveredRelationalRuntimeAuthority,
};

pub(super) fn classify<Schema: ApplicationSchema + 'static>(
    execution: &ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraphBootstrap<Schema>,
    recovered: &RecoveredRelationalRuntimeAuthority,
    basis: &AdmittedRelationalBranchBasis,
    target: &InstalledCheckpointTarget<'_, Schema>,
    checkpoint: &mut DecodedApplicationCheckpoint,
) -> Result<PreservedOutputs, Denial> {
    let mut funding =
        CheckpointMetadataFunding::new(execution).map_err(|e| e.into_installation_denial())?;
    let mut keep = funding
        .vec::<bool>(checkpoint.accepted_outputs.len(), execution)
        .map_err(|e| e.into_installation_denial())?;
    let result = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| {
                classify_rows(
                    execution, graph, recovered, basis, target, checkpoint, &mut keep, context,
                )
            },
        )
    });
    execution.consult().map_err(request)?;
    result.map_err(request)?.map_err(request)?.0?;
    let mut index = 0;
    checkpoint.accepted_outputs.retain_mut(|output| {
        let preserve = keep[index];
        index += 1;
        if preserve {
            // Existing locator-only posture: old facts may not establish target Ready.
            output.producer_facts = None;
            output.producer_fact_wire_version = 0;
        }
        preserve
    });
    PreservedOutputs::retain(&checkpoint.accepted_outputs, execution)
        .map_err(|e| e.into_installation_denial())
}
#[allow(clippy::too_many_arguments)]
fn classify_rows<Schema: ApplicationSchema + 'static>(
    execution: &ExecutionRequest<'_, '_>,
    graph: &WorthQueryPrimaryGraphBootstrap<Schema>,
    recovered: &RecoveredRelationalRuntimeAuthority,
    basis: &AdmittedRelationalBranchBasis,
    target: &InstalledCheckpointTarget<'_, Schema>,
    checkpoint: &DecodedApplicationCheckpoint,
    keep: &mut Vec<bool>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    for output in &checkpoint.accepted_outputs {
        charge(context, 1)?;
        if output.roles.is_empty() {
            return Err(denial(
                "codec recovery cannot classify empty output correspondence",
            ));
        }
        let dependency = output
            .producer_dependency
            .as_ref()
            .ok_or_else(|| denial("codec output lacks original producer dependency"))?;
        // Complete installed role meaning is checked before any performance.
        // This creates only funded typed metadata, never Ready/currentness.
        target
            .producers
            .readmit_checkpoint_output(target.schema, output, execution)
            .map_err(|e| e.into_installation_denial())?;
        let mut role_funding =
            CheckpointMetadataFunding::new(execution).map_err(|e| e.into_installation_denial())?;
        let meaning = target
            .producers
            .checkpoint_publication_meaning(output, &mut role_funding, execution)
            .map_err(|e| e.into_installation_denial())?;
        let mut roles = role_funding
            .vec(output.roles.len(), execution)
            .map_err(|e| e.into_installation_denial())?;
        let mut retired = 0;
        let mut live = 0;
        for role in output.roles.iter() {
            charge(context, role.entity_name.len().saturating_add(1))?;
            let lookup = graph
                .graph
                .layout
                .entity_kind_lookup_work(&role.entity_name)
                .ok_or_else(|| denial("codec output kind lookup work overflow"))?;
            context
                .checkpoint_cost(lookup, lookup)
                .map_err(|stop| request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
            let kind = graph
                .graph
                .layout
                .entity_kind(&role.entity_name)
                .ok_or_else(|| denial("codec output names uninstalled native kind"))?;
            roles.push(worth_relational::facade::mvcc::RequiredOutputRoleTarget {
                role: role_funding.copy_text(&role.role, execution).map_err(|e| e.into_installation_denial())?,
                entity_name: role_funding.copy_text(&role.entity_name, execution).map_err(|e| e.into_installation_denial())?,
                kind, posture: match role.posture {
                    crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture::Create => worth_relational::facade::mvcc::RequiredOutputRolePosture::Create,
                    crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture::Preserve => worth_relational::facade::mvcc::RequiredOutputRolePosture::Preserve,
                    crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture::Retire => worth_relational::facade::mvcc::RequiredOutputRolePosture::Retire,
                },
                target: worth_relational::facade::transactions::EntityReference::Existing(role.entity),
            });
            if role.posture == crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture::Retire {
                continue;
            }
            live += 1;
            let mut is_retired = false;
            let mut matched = false;
            for (entity, native_kind, producer, native_dependency) in
                recovered.codec_transition_output_origins()
            {
                charge(
                    context,
                    producer
                        .len()
                        .saturating_add(output.producer.len())
                        .saturating_add(33),
                )?;
                if entity == role.entity {
                    is_retired = true;
                    matched = native_kind == kind
                        && producer == output.producer.as_str()
                        && dependency == native_dependency;
                    break;
                }
            }
            if is_retired {
                retired += 1;
                if !matched {
                    return Err(denial(
                        "retired output lacks exact performed native producer association",
                    ));
                }
            }
        }
        // Optional role omission cannot hide a mixed native publication. All
        // branches first re-admit the complete original SOURCE publication;
        // none of these descriptions can add a retirement/currentness grant.
        graph
            .graph
            .integration_handle()
            .with_runtime_mut(execution, |runtime| {
                runtime
                    .durability_recovery()
                    .validate_recovered_codec_output_correspondence(
                        execution,
                        recovered,
                        basis,
                        &output.producer,
                        dependency,
                        &meaning,
                        &roles,
                    )
                    .map_err(super::codec::map_admission)
            })?;
        if retired == live && live != 0 {
            keep.push(false);
        } else if retired == 0 {
            keep.push(true);
        } else {
            return Err(denial(
                "codec recovery refuses partially retired output correspondence",
            ));
        }
    }
    Ok(())
}
fn charge(context: &mut MapKernelContext<'_, '_>, work: usize) -> Result<(), Denial> {
    context
        .checkpoint(u64::try_from(work).map_err(|_| denial("codec output work overflow"))?)
        .map_err(|stop| request(worth_execution::WorkCeilingDenial::Stopped(stop)))
}
fn request(cause: worth_execution::WorkCeilingDenial) -> Denial {
    super::super::request_read_denial::read(cause.into())
}
