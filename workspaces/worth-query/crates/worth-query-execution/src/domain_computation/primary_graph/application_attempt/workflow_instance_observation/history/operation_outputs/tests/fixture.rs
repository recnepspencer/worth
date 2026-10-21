//! Uses genuine installed kinds, source identities and World publication, not a workflow authority factory.
mod custody;
mod publication;
use super::super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationCreationPartition as Partition,
        WorthQueryApplicationRealizedEffect as Effect,
    },
    tests::{
        application_attempt::{admitted_operation, authenticated_principal, resolved_account},
        fixture::{
            installed_authorization_world, live_scope, publish_relational_mutation,
            AuthorizationWorld,
        },
    },
    workflow::{actor::record::WorkflowAdmittedActor, operation_output::append_operation_outputs},
};
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as Posture;
use worth_relational::facade::{symbols::ClientKey, transactions::*};

pub(super) struct Context {
    world: AuthorizationWorld,
    pub(super) source: EntityId,
    pub(super) created: EntityId,
    transition: EntityId,
}

impl Context {
    pub(super) fn publish(defect: Option<&str>, retire_source: bool) -> Self {
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
        let batch = publication::batch(&layout, source, actor, account_kind, retire_source, defect);
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

    pub(super) fn check(&self) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.read(|execution, runtime, snapshot, layout| super::super::observe(execution, runtime, snapshot, layout, self.transition,
            &mut super::super::HistoryReconstructionCharge::new(worth_query_installation::facade::WorthQueryWorkflowHistoryReconstructionBudget::standard())?).map(|_| ()))
    }

    pub(super) fn targets(&self) -> Vec<(String, EntityId)> {
        self.read(|execution, runtime, snapshot, layout| {
            let output = &layout.operation_output;
            let mut result = observe_adjacency(runtime, execution, snapshot, output.membership, self.transition, Direction::Outgoing, 8).unwrap().unwrap().into_iter().map(|entry| {
                let role = text(execution, runtime, snapshot, entry.to, output.entity_kind, &output.role).unwrap();
                let target = crate::domain_computation::primary_graph::application_attempt::fact::observe_audit_outgoing_adjacency(runtime, execution, snapshot, output.target, entry.to, 4).unwrap().unwrap()[0].to;
                (role, target)
            }).collect::<Vec<_>>();
            result.sort(); result
        })
    }

    pub(super) fn retire(&self, entity: EntityId) {
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

    fn read<T>(
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

fn inventory(
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
fn text_value(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}

fn payer() -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(64 << 20)),
        worth_execution::CancellationToken::new(),
        None,
    )
}
