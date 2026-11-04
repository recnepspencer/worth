//! Publish fixture activation through World before preparing program-owned work.
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome as PublicationOutcome;

use crate::domain_computation::primary_graph::tests::fixture::product_publication::prepare_relational_mutation_on_application;
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::program_occurrence::revision_rendering::program_revision_rendering;
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::CreatedEntityRef;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

pub fn seed_program_activation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    revision: &ApplicationProgramRevision,
) {
    let layout = (*(*(*(*world
        .application
        .runtime_for_test())
        .primary_provider_for_test())
        .graph_for_test())
        .layout_for_test())
        .program_activation()
        .clone();
    let created = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: (*layout.entity_kind_for_test()),
        client_key: ClientKey::raw("fixture-program-activation"),
    };
    let batch = WorkerIntentBatch::new("seed-program-activation").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: created.partition_id,
            kind_id: created.kind_id,
            client_key: created.client_key.clone(),
            fields: AspectFieldPatch::from(BTreeMap::from([(
                (*layout.program_revision_locator_for_test()).clone(),
                program_revision_rendering(revision),
            )])),
        }),
    ));
    let request = worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
        None,
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new().token(),
        (*(*world.application.product_runtime().clock_for_test()).admission_clock_for_test()).clone(),
    );
    let prepared =
        prepare_relational_mutation_on_application(execution, &world.application, batch, &request);
    let Ok(PublicationOutcome::Performed(performed)) = prepared.execute_for_test(*execution) else {
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
    (*(*(*world
        .application
        .runtime_for_test())
        .primary_provider_for_test())
        .graph_for_test())
        .with_runtime_mut_for_test(execution, |runtime| {
            (*(*(*world
                .application
                .runtime_for_test())
                .primary_provider_for_test())
                .graph_for_test())
                .ensure_primary_indexes_for_basis_for_test(execution, runtime, &basis)
                .unwrap();
        },
    );
    (*world.application.program_support_for_test())
        .as_ref()
        .unwrap()
        .activation()
        .publish(activation)
        .unwrap();
}
