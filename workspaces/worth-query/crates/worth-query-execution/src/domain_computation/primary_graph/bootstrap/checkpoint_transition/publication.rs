//! Installation transition borrows bootstrap custody as a witness; native doors run inline in that request.
//! One target-validation candidate and its acknowledged native successor.
use super::{
    denial, selection, CheckpointTransition, WorthQueryCheckpointMigrationWriter,
    WorthQueryCheckpointTransitionRecovery,
};
use crate::domain_computation::primary_graph::application_installation::WorthQueryInMemoryApplicationDenial;
use crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase;
use crate::domain_computation::primary_graph::{
    application_installation::program_admission::WorthQueryAdmittedProgramSupport,
    bootstrap::program_activation_recovery::read_activation,
    bootstrap_publication::{
        append_typed_rows, map_bootstrap_basis_denial, map_bootstrap_commit_denial,
        map_bootstrap_staging_denial, map_bootstrap_transaction_admission_denial,
    },
    program_occurrence::{program_revision_rendering, WorthQueryProgramActivationCell},
    WorthQueryPrimaryGraphBootstrap,
};
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, RelationMutationIntent,
    RevalidateEntityIntent, RevalidateRelationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

/// Borrowed installed target consumers, never independent migration authority.
pub(in crate::domain_computation::primary_graph) struct InstalledCheckpointTarget<'a, Schema> {
    pub(in crate::domain_computation::primary_graph) schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
    pub(in crate::domain_computation::primary_graph) program: &'a WorthQueryAdmittedProgramSupport<Schema>,
    pub(in crate::domain_computation::primary_graph) producers: &'a crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>,
}

fn prepare_transition<Schema: ApplicationSchema + 'static>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    _phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    checkpoint: &mut crate::domain_computation::primary_graph::application_checkpoint::DecodedApplicationCheckpoint,
    target: &InstalledCheckpointTarget<'_, Schema>,
    cell: &WorthQueryProgramActivationCell,
    transition: CheckpointTransition<'_, '_, Schema>,
) -> Result<
    (
        worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority,
        super::preserved_outputs::PreservedOutputs,
    ),
    WorthQueryInMemoryApplicationDenial,
> {
    let installed = target.schema;
    let support = target.program;
    let layout = graph.graph.layout.clone();
    let mut recovered = graph.recovered_relational_authority.take().ok_or_else(|| {
        WorthQueryInMemoryApplicationDenial::Graph(denial(
            "checkpoint transition has no recovered native authority",
        ))
    })?;
    let codec_pending = recovered.codec_transition_pending();
    let expected = AspectValue::String(InternedString::from(transition.predecessor.0.as_str()));
    let (basis, identity, records) = graph
        .graph
        .integration_handle()
        .with_runtime(execution, |runtime| {
            let basis = runtime
                .admit_branch_basis(&runtime.main_branch_identity())
                .map_err(map_bootstrap_basis_denial)?;
            let branch = runtime
                .read_truth()
                .project_observation(&basis.observation())
                .map_err(|error| {
                    denial(format!(
                        "checkpoint transition source unreadable: {error:?}"
                    ))
                })?;
            let (identity, work) = read_activation(
                execution,
                &branch,
                layout.program_activation(),
                transition.resources.maximum_selection_work,
                "recovered program activation does not match the expected checkpoint predecessor",
                |rendering| *rendering == expected,
            )?;
            let records = selection::select(
                execution,
                &branch,
                &layout,
                transition.resources.maximum_selection_work,
                work,
            )?;
            Ok((basis, identity, records))
        })
        .map_err(WorthQueryInMemoryApplicationDenial::Graph)?;
    let native_transaction = super::codec::begin(
        execution,
        graph,
        &mut recovered,
        &basis,
        worth_relational::facade::runtime::RecoveredCheckpointProgramActivationUpdate {
            entity_id: identity,
            field_locator: layout.program_activation().program_revision_locator.clone(),
            expected,
            replacement: program_revision_rendering(&support.initial_revision),
        },
    )
    .map_err(WorthQueryInMemoryApplicationDenial::Graph)?;
    let preserved_outputs = if codec_pending {
        target
            .producers
            .validate_codec_transition_output_origins(execution, &recovered, &layout)
            .map_err(WorthQueryInMemoryApplicationDenial::Graph)?;
        super::output_disposition::classify(
            execution, graph, &recovered, &basis, target, checkpoint,
        )
        .map_err(WorthQueryInMemoryApplicationDenial::Graph)?
    } else {
        super::preserved_outputs::PreservedOutputs::retain(&checkpoint.accepted_outputs, execution)
            .map_err(|e| WorthQueryInMemoryApplicationDenial::Graph(e.into_installation_denial()))?
    };
    let mut writer = WorthQueryCheckpointMigrationWriter::new(graph, transition.resources);
    let authored = (transition.author)(&mut writer, installed);
    execution.consult().map_err(|cause| {
        WorthQueryInMemoryApplicationDenial::Graph(super::super::request_read_denial::read(
            cause.into(),
        ))
    })?;
    authored.map_err(WorthQueryInMemoryApplicationDenial::Graph)?;
    writer
        .finish()
        .map_err(WorthQueryInMemoryApplicationDenial::Graph)?;
    if codec_pending && (!graph.entity_rows.is_empty() || !graph.relation_rows.is_empty()) {
        return Err(WorthQueryInMemoryApplicationDenial::Graph(denial(
            "codec-only recovery cannot append unclassified authored rows",
        )));
    }
    cell.bind_checkpoint_candidate(identity).map_err(|_| {
        WorthQueryInMemoryApplicationDenial::Graph(denial(
            "checkpoint activation candidate was already bound",
        ))
    })?;
    let mut batch = WorkerIntentBatch::new("application-checkpoint-program-transition");
    if !codec_pending {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: identity,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    layout.program_activation().program_revision_locator.clone(),
                    program_revision_rendering(&support.initial_revision),
                )])),
            },
        )));
    }
    batch = append_typed_rows(
        batch,
        std::mem::take(&mut graph.entity_rows),
        std::mem::take(&mut graph.relation_rows),
    );
    for entity_id in records.entities {
        if recovered.codec_transition_retires(entity_id) {
            continue;
        }
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Revalidate(
            RevalidateEntityIntent { entity_id },
        )));
    }
    for relation_id in records.relations {
        if recovered.codec_transition_retires_relation(relation_id) {
            continue;
        }
        batch = batch.push(MutationIntent::Relation(
            RelationMutationIntent::Revalidate(RevalidateRelationIntent { relation_id }),
        ));
    }
    transition.capture_policy.check_live().map_err(|error| {
        WorthQueryInMemoryApplicationDenial::CheckpointTransitionPolicyStopped(error.into())
    })?;
    let successor = graph
        .graph
        .integration_handle()
        .with_runtime_mut(execution, |runtime| {
            let mut transaction = match native_transaction {
                Some(transaction) => transaction,
                None => runtime
                    .begin_branch_transaction(
                        &basis,
                        worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                    )
                    .map_err(map_bootstrap_transaction_admission_denial)?,
            };
            transaction
                .push_batch(
                    batch,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .map_err(map_bootstrap_staging_denial)?;
            let candidate = runtime
                .prepare_branch_transaction(execution, transaction)
                .map_err(map_bootstrap_commit_denial)?;
            Ok(runtime
                .durability_recovery()
                .commit_checkpoint_transition(execution, recovered, candidate))
        })
        .map_err(WorthQueryInMemoryApplicationDenial::Graph)?;
    match successor {
        Ok(acknowledged) => Ok((acknowledged.into_parts().1, preserved_outputs)),
        Err(worth_relational::facade::durability::RecoveredCheckpointTransitionError::Refused(refusal)) => Err(WorthQueryInMemoryApplicationDenial::Graph(super::codec::map_refusal(refusal.denial()))),
        Err(worth_relational::facade::durability::RecoveredCheckpointTransitionError::SettlementFailed(error)) => Err(WorthQueryInMemoryApplicationDenial::CheckpointTransitionSettlementFailed(error)),
    }
}

pub(in crate::domain_computation::primary_graph) fn transition_checkpoint<
    Schema: ApplicationSchema + 'static,
>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    checkpoint: &mut crate::domain_computation::primary_graph::application_checkpoint::DecodedApplicationCheckpoint,
    target: InstalledCheckpointTarget<'_, Schema>,
    cell: &WorthQueryProgramActivationCell,
    transition: CheckpointTransition<'_, '_, Schema>,
) -> Result<(), WorthQueryInMemoryApplicationDenial> {
    let phase_request = phase.execution_request();
    let execution = &phase_request;
    let recovery = std::rc::Rc::clone(&transition.recovery);
    let capture_policy = transition.capture_policy;
    let codec_pending = graph
        .recovered_relational_authority
        .as_ref()
        .is_some_and(|authority| authority.codec_transition_pending());
    let (successor, preserved_outputs) = prepare_transition(
        execution, phase, graph, checkpoint, &target, cell, transition,
    )?;
    let publication = graph
        .recovered_publication
        .clone()
        .expect("transition holds its original native bootstrap publication");
    if codec_pending && let Err(cause) = super::codec::finish(execution, graph) {
        return Err(
            WorthQueryInMemoryApplicationDenial::CheckpointTransitionFinalizationStopped(Box::new(
                WorthQueryCheckpointTransitionRecovery::finalization_stopped(
                    graph.graph.integration_handle(),
                    publication,
                    successor,
                    preserved_outputs,
                    cause,
                ),
            )),
        );
    }
    match capture_policy
        .check_live()
        .map_err(crate::domain_computation::primary_graph::WorthQueryCheckpointCaptureDenial::from)
        .and_then(|()| {
            graph
                .graph
                .integration_handle()
                .with_runtime(execution, |runtime| runtime.durability_authority().native_checkpoint(capture_policy))
                .map_err(crate::domain_computation::primary_graph::WorthQueryCheckpointCaptureDenial::from)
        })
        .and_then(|native| {
            crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint::encode(
                native,
                &publication,
                preserved_outputs.rows(),
                capture_policy,
            )
            .map(|(checkpoint, _)| checkpoint)
        }) {
        Ok(checkpoint) => *recovery.borrow_mut() = Some(checkpoint),
        Err(error) => {
            return Err(
                WorthQueryInMemoryApplicationDenial::CheckpointTransitionCaptureStopped(Box::new(
                    WorthQueryCheckpointTransitionRecovery::acknowledged(
                        graph.graph.integration_handle(),
                        publication,
                        successor,
                        preserved_outputs,
                        error,
                    ),
                )),
            )
        }
    }
    graph.recovered_relational_authority = Some(successor);
    cell.confirm_checkpoint_candidate().map_err(|_| {
        WorthQueryInMemoryApplicationDenial::Graph(denial(
            "checkpoint activation confirmation refused",
        ))
    })?;
    Ok(())
}
