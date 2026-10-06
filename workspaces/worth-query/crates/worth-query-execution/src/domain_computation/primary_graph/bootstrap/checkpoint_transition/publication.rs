//! One target-validation candidate and its acknowledged native successor.
use super::{
    denial, selection, WorthQueryOpenAdoption, WorthQueryOpenAdoptionRecovery,
    WorthQueryOpenAdoptionWriter,
};
use crate::domain_computation::primary_graph::application_installation::{
    OpenFailure, WorthQueryApplicationOpenDenial,
};
use crate::domain_computation::primary_graph::{
    application_installation::program_admission::WorthQueryAdmittedProgramSupport,
    bootstrap::program_activation_recovery::read_activation,
    bootstrap_publication::{
        append_typed_rows, map_bootstrap_basis_denial, map_bootstrap_staging_denial,
        map_bootstrap_transaction_admission_denial,
    },
    program_occurrence::{program_revision_rendering, WorthQueryProgramActivationCell},
    WorthQueryApplicationCheckpoint, WorthQueryPrimaryGraphBootstrap,
};
use std::collections::BTreeMap;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};
use worth_relational::facade::durability::RecoveredCheckpointTransitionError;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, RevalidateEntityIntent,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};

fn prepare_transition<Schema: ApplicationSchema>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    support: &WorthQueryAdmittedProgramSupport<Schema>,
    cell: &WorthQueryProgramActivationCell,
    adoption: WorthQueryOpenAdoption<'_, Schema>,
) -> Result<worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority, OpenFailure>
{
    let layout = graph.graph.layout.clone();
    let expected = adoption.predecessor.rendering();
    let (basis, identity, entities) = graph
        .graph
        .integration_handle()
        .with_runtime(|runtime| {
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
                &branch,
                layout.program_activation(),
                adoption.resources.maximum_selection_work,
                "recovered program activation does not match the expected checkpoint predecessor",
                |rendering| *rendering == expected,
            )?;
            let entities = selection::select(
                &branch,
                &layout,
                installed,
                adoption.resources.maximum_selection_work,
                work,
            )?;
            Ok((basis, identity, entities))
        })
        .map_err(WorthQueryApplicationOpenDenial::Graph)?;
    let mut writer = WorthQueryOpenAdoptionWriter::new(graph, adoption.resources);
    (adoption.author)(&mut writer, installed).map_err(WorthQueryApplicationOpenDenial::Graph)?;
    writer
        .finish()
        .map_err(WorthQueryApplicationOpenDenial::Graph)?;
    cell.bind_checkpoint_candidate(identity).map_err(|_| {
        WorthQueryApplicationOpenDenial::Graph(denial(
            "checkpoint activation candidate was already bound",
        ))
    })?;
    let mut batch = WorkerIntentBatch::new("application-checkpoint-program-transition").push(
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: identity,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    layout.program_activation().program_revision_locator.clone(),
                    program_revision_rendering(&support.initial_revision),
                )])),
            },
        )),
    );
    batch = append_typed_rows(
        batch,
        std::mem::take(&mut graph.entity_rows),
        std::mem::take(&mut graph.relation_rows),
    );
    for entity_id in entities {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Revalidate(
            RevalidateEntityIntent { entity_id },
        )));
    }
    let recovered = graph.recovered_relational_authority.take().ok_or_else(|| {
        WorthQueryApplicationOpenDenial::Graph(denial(
            "checkpoint transition has no recovered native authority",
        ))
    })?;
    let successor = graph
        .graph
        .integration_handle()
        .with_runtime_mut(|runtime| {
            let mut transaction = runtime
                .begin_branch_transaction(
                    &basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .map_err(map_bootstrap_transaction_admission_denial)?;
            transaction
                .push_batch(batch)
                .map_err(map_bootstrap_staging_denial)?;
            let candidate = runtime
                .prepare_branch_transaction(transaction)
                .map_err(|error| {
                    denial(format!("checkpoint target validation refused: {error:?}"))
                })?;
            Ok(runtime
                .durability_recovery()
                .commit_checkpoint_transition(recovered, candidate))
        })
        .map_err(WorthQueryApplicationOpenDenial::Graph)?;
    match successor {
        Ok(acknowledged) => Ok(acknowledged.into_parts().1),
        Err(RecoveredCheckpointTransitionError::DurabilityDeferred(deferred)) => {
            let publication = graph
                .recovered_publication
                .clone()
                .expect("native checkpoint bootstrap publication was readmitted before transition");
            let recovery = WorthQueryOpenAdoptionRecovery::new(
                graph.graph.integration_handle(),
                publication,
                *deferred,
            );
            let cause = recovery.detail().to_owned();
            Err(OpenFailure::in_repair(
                recovery,
                WorthQueryApplicationOpenDenial::AdoptionDeferred(cause),
            ))
        }
        Err(RecoveredCheckpointTransitionError::Refused(refusal)) => {
            Err(WorthQueryApplicationOpenDenial::Graph(denial(format!(
                "checkpoint native transition refused without performance: {:?}",
                refusal.denial()
            )))
            .into())
        }
        Err(RecoveredCheckpointTransitionError::SettlementFailed(error)) => {
            Err(WorthQueryApplicationOpenDenial::AdoptionSettlementFailed(error).into())
        }
    }
}

/// Performs the declared adoption and returns the acknowledged successor image.
///
/// A refusal before native performance leaves the home unchanged. A deferred
/// settlement or a stopped capture leaves only the repair capsule. A refusal
/// after capture leaves the successor image.
pub(in crate::domain_computation::primary_graph) fn transition_checkpoint<
    Schema: ApplicationSchema,
>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    support: &WorthQueryAdmittedProgramSupport<Schema>,
    cell: &WorthQueryProgramActivationCell,
    adoption: WorthQueryOpenAdoption<'_, Schema>,
) -> Result<WorthQueryApplicationCheckpoint, OpenFailure> {
    let successor = prepare_transition(graph, installed, support, cell, adoption)?;
    let publication = graph
        .recovered_publication
        .clone()
        .expect("transition holds its original native bootstrap publication");
    let image = match graph
        .graph
        .integration_handle()
        .with_runtime(|runtime| runtime.durability_authority().native_checkpoint())
    {
        Ok(native) => WorthQueryApplicationCheckpoint::encode(native, &publication, &[]).0,
        Err(error) => {
            let cause = format!("acknowledged target checkpoint capture stopped: {error:?}");
            return Err(OpenFailure::in_repair(
                WorthQueryOpenAdoptionRecovery::acknowledged(
                    graph.graph.integration_handle(),
                    publication,
                    successor,
                    cause.clone(),
                ),
                WorthQueryApplicationOpenDenial::AdoptionCaptureStopped(cause),
            ));
        }
    };
    graph.recovered_relational_authority = Some(successor);
    if cell.confirm_checkpoint_candidate().is_err() {
        return Err(OpenFailure::successor(
            image,
            WorthQueryApplicationOpenDenial::Graph(denial(
                "checkpoint activation confirmation refused",
            )),
        ));
    }
    Ok(image)
}
