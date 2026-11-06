//! A merge writes unique values under the program law: each value it writes
//! is looked up at the target head and must be free or held only by the
//! entity the merge writes, and one merge writes each value at most once.

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
use worth_relational::facade::merge::MergeExecutionRequest;
use worth_relational::facade::merge::MergeIntent;
use worth_relational::facade::merge::PreparedMergeExecution;
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
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::merge_unique_values::admit_merge_unique_values;
use worth_query_execution::facade::integration::WorthQueryMergeUniqueValueDenialKind as Kind;
use worth_query_execution::court_doors_for_test::domain_computation::graph_layout::unique_fields::WorthQueryUniqueFieldFixture;
use worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle;
pub const FEATURE: &str = "unique-feature";
pub struct MergeWorld {
    pub _world: IdentityWorld,
    pub handle: WorthQueryPrimaryGraphIntegrationHandle,
    pub kind: KindId,
    pub label: AspectFieldLocator,
    pub status: AspectFieldLocator,
    /// The fields a create must carry besides the two it varies.
    pub required: [AspectFieldLocator; 3],
    pub index: DerivedIndexId,
}
impl MergeWorld {
    pub fn new() -> Self {
        let world = installed_world(&[("merge-unique", WorthQueryPrincipalMappingStatus::Enabled)]);
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
        let label = field("AccountPolicy", "AccountLabel");
        let status = field("AccountPolicy", "AccountStatus");
        let required = [
            field("AccountPolicy", "AccountIdentity").locator_into_for_test(),
            field("AccountMembership", "AccountMembershipTag").locator_into_for_test(),
            field("AccountAnnotations", "AccountAnnotation").locator_into_for_test(),
        ];
        Self {
            kind: (*label.entity_kind_for_test()),
            index: (*label.equality_index_id_for_test()).expect("the label is equality-indexed"),
            label: label.locator_into_for_test(),
            status: status.locator_into_for_test(),
            required,
            handle,
            _world: world,
        }
    }

    pub fn unique(&self, index: Option<DerivedIndexId>) -> WorthQueryUniqueFieldFixture {
        WorthQueryUniqueFieldFixture::new_for_test(self.kind, self.label.clone(), index)
    }

    pub fn fork(&self) {
        let host_request = crate::test_request_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        self.mutate(execution, |runtime| {
            let (_, source) = runtime
                .observe_fork_source(&BranchId("main".to_owned()))
                .expect("main exposes a fork source");
            runtime
                .fork_branch(execution, BranchId(FEATURE.to_owned()), source)
                .expect("the feature branch forks");
        });
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

    /// Admits the feature-into-main merge under the installed schema, and
    /// executes it when admitted.
    pub fn merge(&self, execution: &worth_execution::ExecutionRequest<'_, '_>) -> Result<(), Kind> {
        let unique = self.unique(Some(self.index));
        self.mutate(execution, |runtime| {
            let prepared = prepared_merge(execution, runtime);
            admit_merge_unique_values(unique.fields(), runtime, execution, &prepared)
                .map_err(|denial| denial.kind())?;
            runtime
                .execute_prepared_merge(execution, prepared)
                .expect("an admitted merge executes");
            Ok(())
        })
    }

    pub fn created_fields(&self, key: &str, label: u64) -> AspectFieldPatch {
        let [identity, membership, annotation] = self.required.clone();
        AspectFieldPatch::from(BTreeMap::from([
            (self.label.clone(), value(label)),
            (self.status.clone(), value(1)),
            (
                identity,
                AspectValue::String(InternedString::from(key.to_owned())),
            ),
            (membership, value(1)),
            (annotation, value(1)),
        ]))
    }

    pub fn mutate<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> T {
        self.handle
            .execute_mutation_with_index_refresh(execution, |runtime| Ok::<_, ()>(mutate(runtime)))
            .expect("the unique field's index refreshes")
            .expect("the fixture mutation is infallible")
    }
}
pub fn prepared_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
) -> PreparedMergeExecution {
    let bound = runtime
        .bind_merge_execution_request(merge_request())
        .unwrap();
    runtime.prepare_merge_execution(execution, bound).unwrap()
}
pub fn merge_request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".to_owned()),
        BranchId(FEATURE.to_owned()),
        MergeIntent::ReconcileIntoTarget,
    )
}
pub fn commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
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
            WorkerIntentBatch::new("unique-merge").push(intent),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the fixture batch stages");
    transaction
        .commit(execution, runtime)
        .expect("the fixture commits")
}
pub fn value(ordinal: u64) -> AspectValue {
    AspectValue::String(InternedString::from(format!("value-{ordinal}")))
}
