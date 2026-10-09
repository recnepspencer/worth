//! Publish fixture activation through World before preparing program-owned work.

use super::{prepare_relational_mutation_on_application, AuthorizationWorld};
use crate::domain_computation::primary_graph::program_occurrence::program_revision_rendering;
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent,
        WorkerIntentBatch,
    },
};

pub(in crate::domain_computation::primary_graph) fn seed_program_activation(
    world: &AuthorizationWorld,
    revision: &ApplicationProgramRevision,
) {
    let layout = world
        .application
        .primary_provider
        .graph
        .layout
        .program_activation()
        .clone();
    let created = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: layout.entity_kind,
        client_key: ClientKey::raw("fixture-program-activation"),
    };
    let batch = WorkerIntentBatch::new("seed-program-activation").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: created.partition_id,
            kind_id: created.kind_id,
            client_key: created.client_key.clone(),
            fields: AspectFieldPatch::from(BTreeMap::from([(
                layout.program_revision_locator.clone(),
                program_revision_rendering(revision),
            )])),
        }),
    ));
    let request = super::live_scope();
    let prepared = prepare_relational_mutation_on_application(&world.application, batch, &request);
    let worth_runtime_world::facade::RuntimeWorldPublicationOutcome::Performed(performed) = world
        .application
        .with_application_advancement(&request, |phase| {
            prepared.execute(
                phase
                    .execution_request_for(world.application.product_runtime())
                    .unwrap(),
            )
        })
        .unwrap()
    else {
        panic!("activation seed must publish through World");
    };
    let activation = performed
        .component_results()
        .relational_commit_result()
        .unwrap()
        .created_entity(&created)
        .unwrap();
    let basis = performed.commit().basis().relational_basis().clone();
    drop(performed.consume());
    world
        .application
        .primary_provider
        .graph
        .with_runtime_mut(|runtime| {
            world
                .application
                .primary_provider
                .graph
                .ensure_primary_indexes_for_basis(runtime, &basis)
                .unwrap();
        });
    world
        .application
        .program_support
        .as_ref()
        .unwrap()
        .activation()
        .publish(activation)
        .unwrap();
}
