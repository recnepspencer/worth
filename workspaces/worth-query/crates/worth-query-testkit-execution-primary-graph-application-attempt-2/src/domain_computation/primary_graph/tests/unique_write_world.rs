//! An installed Native world with a unique application field.

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::storage_model::{Account, AccountPolicy, AccountPolicyValue, AccountMembership, AccountMembershipValue, AccountAnnotations, AccountAnnotationsValue, AnnotationText, ExecutionRecords, ExecutionRecordsBindings, AccountLabel, AccountStatus, AccountIdentity, AccountMembershipTag, AccountAnnotation};
use std::collections::BTreeMap;
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::InternedString;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::CreatedEntityRef;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::IdentityWorld;
use worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle;
pub struct UniqueWriteWorld {
    pub _world: IdentityWorld,
    pub handle: WorthQueryPrimaryGraphIntegrationHandle,
    pub kind: KindId,
    pub label: AspectFieldLocator,
    pub index: DerivedIndexId,
}
impl Default for UniqueWriteWorld {
    fn default() -> Self {
        Self::new()
    }
}
impl UniqueWriteWorld {
    pub fn new() -> Self {
        let world = installed_world(&[("unique-write", WorthQueryPrincipalMappingStatus::Enabled)]);
        let handle = (*world.application.runtime_for_test())
            .primary_graph_for_test()
            .expect("the fixture publishes its graph")
            .integration_handle_for_test();
        let field = |aspect: &str, name: &str| {
            (*handle.layout_for_test())
                .field_for_test("Account", aspect, name)
                .expect("the schema declares the field")
                .clone()
        };
        let label = field(AccountLabel::at("AccountLabel").aspect(), "AccountLabel");
        Self {
            kind: (*label.entity_kind_for_test()),
            index: (*label.equality_index_id_for_test()).expect("the label is equality-indexed"),
            label: label.locator_into_for_test(),
            handle,
            _world: world,
        }
    }

    pub fn create(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &str,
        key: &str,
        handle: u64,
    ) -> EntityId {
        let created = CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: self.kind,
            client_key: ClientKey::raw(key),
        };
        let intent = MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: created.partition_id,
            kind_id: created.kind_id,
            client_key: created.client_key.clone(),
            fields: self.created_fields(key, handle),
        }));
        self.mutate(execution, |runtime| {
            let committed = commit(execution, runtime, branch, intent);
            let entity = committed
                .created_entity(&created)
                .expect("the create issued");
            worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
                runtime,
                &committed.snapshot,
            );
            entity
        })
    }

    pub fn created_fields(&self, key: &str, label: u64) -> AspectFieldPatch {
        let fragment = self
            ._world
            .application
            .installed_schema()
            .value_bindings()
            .fragment(ExecutionRecords)
            .unwrap();
        let policy = fragment.aspect::<Account, AccountPolicy>().unwrap();
        let membership = fragment.aspect::<Account, AccountMembership>().unwrap();
        let annotations = fragment.aspect::<Account, AccountAnnotations>().unwrap();
        let rows = [
            (
                policy,
                policy
                    .encode::<ExecutionRecordsBindings, _>(&AccountPolicyValue {
                        account_identity: key.to_owned(),
                        account_status: "value-1".to_owned(),
                        account_label: format!("value-{label}"),
                        account_note: None,
                        account_score: None,
                    })
                    .unwrap(),
            ),
            (
                membership,
                membership
                    .encode::<ExecutionRecordsBindings, _>(&AccountMembershipValue {
                        account_membership_tag: "value-1".to_owned(),
                    })
                    .unwrap(),
            ),
            (
                annotations,
                annotations
                    .encode::<ExecutionRecordsBindings, _>(&AccountAnnotationsValue {
                        account_annotation: Some(AnnotationText::from("value-1")),
                    })
                    .unwrap(),
            ),
        ];
        let mut values = BTreeMap::new();
        for (definition, mut encoded) in rows {
            for field in definition.fields() {
                if let Some(value) = encoded.remove(field.path()) {
                    let locus = field.recipe().locus();
                    let locator = self
                        .handle
                        .layout_for_test()
                        .field_for_test(locus.entity(), locus.aspect(), locus.field())
                        .unwrap()
                        .locator_for_test()
                        .clone();
                    values.insert(locator, value);
                }
            }
        }
        AspectFieldPatch::from(values)
    }

    pub fn mutate<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutate: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T {
        self.handle
            .execute_mutation_with_index_refresh(execution, |runtime| Ok::<_, ()>(mutate(runtime)))
            .expect("the unique field's index refreshes")
            .expect("the fixture mutation is infallible")
    }
}

pub fn commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &str,
    intent: MutationIntent,
) -> worth_relational::facade::transactions::CommitResult {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("the branch exists");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("the branch admits");
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("the fixture transaction begins");
    transaction
        .push_batch(
            WorkerIntentBatch::new("unique-write").push(intent),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the fixture batch stages");
    transaction
        .commit(execution, runtime)
        .expect("the fixture commits")
}
