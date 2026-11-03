//! Uses genuine installed kinds, source identities and World publication, not a workflow authority factory.

pub use worth_foundational::facade::AspectValue;
pub use std::collections::BTreeMap;
pub use worth_query_execution::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationAdjacencyDirection as Direction;
pub use worth_relational::facade::identity::EntityId;
pub use worth_foundational::facade::InternedString;
pub use worth_relational::facade::identity::KindId;
pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as Posture;
pub use worth_relational::facade::runtime::RelationalRuntime;
pub use worth_relational::facade::snapshots::SnapshotHandle;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenial;
pub use worth_query_execution::domain_computation::primary_graph::workflow::schema::layout::WorthQueryWorkflowLayout;
pub use worth_query_execution::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_adjacency;
pub use crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::history::operation_outputs::tests;
pub use worth_query_execution::domain_computation::primary_graph::application_attempt::workflow_instance_observation::history::operation_outputs::text;
pub use worth_query_execution::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationCreationPartition as Partition;
pub use worth_query_execution::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationRealizedEffect as Effect;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_operation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_execution::domain_computation::primary_graph::workflow::actor::record::WorkflowAdmittedActor;
pub use worth_query_execution::domain_computation::primary_graph::workflow::operation_output::publication::append_operation_outputs;
pub use worth_relational::facade::symbols::ClientKey;
pub use worth_relational::facade::transactions::*;

pub struct Context {
    pub world: AuthorizationWorld,
    pub source: EntityId,
    pub created: EntityId,
    pub transition: EntityId,
}

impl Context {
    pub fn publish(defect: Option<&str>, retire_source: bool) -> Self {
        let world = installed_authorization_world(true);
        let host = payer();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let (source, actor) = world
            .application
            .with_host_advancement(|phase| {
                let request = live_scope();
                let principal =
                    authenticated_principal(&phase.test_host_request(), &world, &request);
                let scope = resolved_account(&phase.test_host_request(), &world, "open", &request);
                let admitted = admitted_operation(&phase, &world, &principal, &scope, &request);
                (
                    admitted.scope_entity_id(),
                    WorkflowAdmittedActor::from_admission(&admitted),
                )
            })
            .unwrap();
        let graph = world.application.runtime.primary_graph().unwrap();
        let layout = graph.layout.workflow().clone();
        let account_kind = graph.layout.entity_kind("Account").unwrap();
        let before = inventory(&world, &execution, account_kind);
        let batch = crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::history::operation_outputs::tests::fixture::publication::batch(&layout, source, actor, account_kind, retire_source, defect);
        publish_relational_mutation(&execution, &world, batch);
        let created = inventory(&world, &execution, account_kind)
            .into_iter()
            .find(|id| !before.contains(id))
            .unwrap();
        let transition = inventory(&world, &execution, layout.transition.entity_kind)
            .pop()
            .unwrap();
        Self {
            world,
            source,
            created,
            transition,
        }
    }

    pub fn check(&self) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.read(|execution, runtime, snapshot, layout| worth_query_execution::domain_computation::primary_graph::application_attempt::workflow_instance_observation::history::operation_outputs::observe(execution, runtime, snapshot, layout, self.transition,
            &mut worth_query_execution::domain_computation::primary_graph::application_attempt::workflow_instance_observation::history::budget::HistoryReconstructionCharge::new(worth_query_installation::facade::WorthQueryWorkflowHistoryReconstructionBudget::standard())?).map(|_| ()))
    }

    pub fn targets(&self) -> Vec<(String, EntityId)> {
        self.read(|execution, runtime, snapshot, layout| {
            let output = &layout.operation_output;
            let mut result = observe_adjacency(runtime, execution, snapshot, output.membership, self.transition, Direction::Outgoing, 8).unwrap().unwrap().into_iter().map(|entry| {
                let role = text(execution, runtime, snapshot, entry.to, output.entity_kind, &output.role).unwrap();
                let target = worth_query_execution::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_audit_outgoing_adjacency(runtime, execution, snapshot, output.target, entry.to, 4).unwrap().unwrap()[0].to;
                (role, target)
            }).collect::<Vec<_>>();
            result.sort(); result
        })
    }

    pub fn retire(&self, entity: EntityId) {
        let host = payer();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        publish_relational_mutation(
            &execution,
            &self.world,
            WorkerIntentBatch::new("retire-business-output").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: entity }),
            )),
        );
    }

    pub fn read<T>(
        &self,
        read: impl FnOnce(
            &worth_execution::ExecutionRequest<'_, '_>,
            &RelationalRuntime,
            &SnapshotHandle,
            &WorthQueryWorkflowLayout,
        ) -> T,
    ) -> T {
        let host = payer();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let (_, _, lease) = self
            .world
            .application
            .select_product_branch(
                &execution,
                self.world.application.product_runtime().default_branch(),
            )
            .unwrap()
            .into_parts();
        let graph = self.world.application.runtime.primary_graph().unwrap();
        graph
            .integration_handle()
            .with_runtime(&execution, |runtime| {
                read(
                    &execution,
                    runtime,
                    lease.snapshot_handle(),
                    graph.layout.workflow(),
                )
            })
    }
}

pub fn inventory(
    world: &AuthorizationWorld,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    kind: KindId,
) -> Vec<EntityId> {
    let (_, _, lease) = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap()
        .into_parts();
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            runtime
                .read_truth()
                .visible_entities_of_kind(execution, kind, lease.snapshot_handle().version_id())
                .unwrap()
                .into_iter()
                .map(|row| row.entity_id)
                .collect()
        })
}
pub fn text_value(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}

pub fn payer() -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(64 << 20)),
        worth_execution::CancellationToken::new(),
        None,
    )
}

#[path = "custody/mod.rs"]
pub mod custody;
#[path = "publication/mod.rs"]
pub mod publication;
