//! Native Relational writer that is not Query, for subscription certification.

use std::collections::BTreeMap;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationScalarValueBinding, ApplicationSchema,
    DeclaredApplicationFieldValue,
};
use worth_relational::facade::{
    identity::{EntityId, PartitionId},
    indexes::DerivedIndexBuildRequest,
    transactions::{
        AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};
use worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts;
use worth_runtime_world::facade::RuntimeWorldPublicationOutcome;

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::basis::WorthQueryProductBranch;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Publishes one field value through an ordinary native Relational
    /// transaction on `branch`, bypassing every Query commit path.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn publish_native_field_write_for_test<
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
    >(
        &self,
        branch: WorthQueryProductBranch,
        record: RelationalBridgeRecordIdentityParts,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
        request: &WorthQueryRequestScope,
    ) where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.with_application_advancement(request, |phase| {
        let value = Field::Binding::encode(&value).expect("the native field value encodes");
        let entity_id = EntityId::new(
            PartitionId(record.partition_id()),
            record.local_slot(),
            record.generation(),
        );
        let selected = self
            .on_branch(branch)
            .select()
            .expect("the native writer branch remains admitted");
        let (_, product, application_basis) = selected.into_parts();
        let graph = self
            .runtime
            .primary_graph()
            .expect("the application retains its primary graph");
        let handle = graph.integration_handle();
        let candidate = handle.with_query_runtime_mut(|runtime, layout| {
            let locator = layout
                .field_locator(field.entity(), field.aspect(), field.field())
                .expect("the native field is installed")
                .clone();
            let batch = WorkerIntentBatch::new("native-field-write").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id,
                    fields: AspectFieldPatch::from(BTreeMap::from([(locator, value)])),
                }),
            ));
            let mut transaction = runtime
                .begin_branch_transaction(
                    product.relational_basis(),
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("the selected product basis admits the native transaction");
            transaction
                .push_batch(
                    batch,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("the native field write stays within resource budgets");
            runtime
                .prepare_branch_transaction(
                    transaction,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("the native field write prepares a Relational candidate")
        });
        let outcome = product
            .publication_binding()
            .prepare_relational_candidate(candidate, request, false)
            .expect("the native World publication prepares")
            .execute(phase.execution_request_for(&self.product_runtime).expect("private progression uses its admitted runtime phase"));
        let RuntimeWorldPublicationOutcome::Performed(performed) = outcome else {
            panic!("the native World publication must perform: {outcome:?}");
        };
        let published = performed.commit().basis().relational_basis().clone();
        drop(performed.consume());
        handle.with_runtime_mut(|runtime| {
            let observation = published.observation();
            let head = observation
                .commit_receipt()
                .expect("the native publication has a canonical commit");
            crate::domain_computation::primary_graph::index_maintenance_budget::refresh_with_cold_fallback(
                runtime,
                DerivedIndexBuildRequest {
                    source_commit_id: head.commit_id,
                    branch_id: head.branch_id.clone(),
                    index_ids: handle.primary_index_ids.to_vec(),
                },
                &published,
                Some(application_basis.snapshot_handle()),
            )
            .expect("the native publication refreshes its exact indexes");
        });
        }).expect("the declared native writer policy admits its host call");
    }
}
