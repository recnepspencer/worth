//! A merge writes unique values under the program law: each value it writes
//! is looked up at the target head and must be free or held only by the
//! entity the merge writes, and one merge writes each value at most once.

use std::collections::BTreeMap;

use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::merge::{MergeExecutionRequest, MergeIntent, PreparedMergeExecution};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent, WorkerIntentBatch,
};

use super::fixture::{installed_world, IdentityWorld};
use crate::domain_computation::primary_graph::merge_unique_values::{
    admit_merge_unique_values, WorthQueryMergeUniqueValueDenialKind as Kind,
};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryUniqueFieldFixture;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;

const FEATURE: &str = "unique-feature";

#[test]
fn a_merge_that_creates_a_value_the_target_holds_is_denied() {
    let world = MergeWorld::new();
    world.fork();
    world.create("main", "ada", 7);
    world.create(FEATURE, "bob", 7);
    assert_eq!(world.merge(), Err(Kind::ValueTaken));
}

#[test]
fn a_merge_of_free_values_commits() {
    let world = MergeWorld::new();
    world.fork();
    world.create("main", "ada", 7);
    world.create(FEATURE, "bob", 8);
    world.create(FEATURE, "cy", 9);
    assert_eq!(world.merge(), Ok(()));
}

#[test]
fn one_merge_writes_each_unique_value_at_most_once() {
    let world = MergeWorld::new();
    world.fork();
    world.create(FEATURE, "ada", 7);
    world.create(FEATURE, "bob", 7);
    assert_eq!(world.merge(), Err(Kind::ValueTaken));
}

#[test]
fn an_uninstalled_unique_index_is_its_own_denial() {
    let world = MergeWorld::new();
    world.fork();
    world.create(FEATURE, "ada", 7);
    let unindexed = world.unique(None);
    let admitted = world.mutate(|runtime| {
        let prepared = prepared_merge(runtime);
        admit_merge_unique_values(unindexed.fields(), runtime, &prepared)
            .map_err(|denial| denial.kind())
    });
    assert_eq!(admitted, Err(Kind::IndexUnavailable));
}

/// More holders than the lookup's candidate limit hold the value: a merge
/// that writes it reads it taken, never unavailable.
#[test]
fn a_value_held_past_the_candidate_limit_reads_as_taken() {
    let world = MergeWorld::new();
    for key in ["ada", "ada-restored", "ada-restored-again"] {
        world.create("main", key, 7);
    }
    world.fork();
    world.create(FEATURE, "bob", 7);
    assert_eq!(world.merge(), Err(Kind::ValueTaken));
}

#[test]
fn restored_duplicates_deny_merges_of_their_value_only() {
    let world = MergeWorld::new();
    world.create("main", "ada", 7);
    world.create("main", "ada-restored", 7);
    world.fork();
    world.create(FEATURE, "bob", 8);
    assert_eq!(world.merge(), Ok(()));
    world.create(FEATURE, "cy", 7);
    assert_eq!(world.merge(), Err(Kind::ValueTaken));
}

/// An installed graph whose `Account.AccountLabel`, an equality-indexed
/// field, is read as unique; raw commits let branches hold what a program
/// would be refused.
struct MergeWorld {
    _world: IdentityWorld,
    handle: WorthQueryPrimaryGraphIntegrationHandle,
    kind: KindId,
    label: AspectFieldLocator,
    status: AspectFieldLocator,
    /// The fields a create must carry besides the two it varies.
    required: [AspectFieldLocator; 3],
    index: DerivedIndexId,
}

impl MergeWorld {
    fn new() -> Self {
        let world = installed_world(&[("merge-unique", WorthQueryPrincipalMappingStatus::Enabled)]);
        let handle = world
            .application
            .runtime
            .primary_graph()
            .expect("the fixture publishes its graph")
            .integration_handle();
        let field = |aspect: &str, name: &str| {
            handle
                .layout
                .field("Account", aspect, name)
                .expect("the schema declares the field")
                .clone()
        };
        let label = field("AccountPolicy", "AccountLabel");
        let status = field("AccountPolicy", "AccountStatus");
        let required = [
            field("AccountPolicy", "AccountIdentity").locator,
            field("AccountMembership", "AccountMembershipTag").locator,
            field("AccountAnnotations", "AccountAnnotation").locator,
        ];
        Self {
            kind: label.entity_kind,
            index: label
                .equality_index_id
                .expect("the label is equality-indexed"),
            label: label.locator,
            status: status.locator,
            required,
            handle,
            _world: world,
        }
    }

    fn unique(&self, index: Option<DerivedIndexId>) -> WorthQueryUniqueFieldFixture {
        WorthQueryUniqueFieldFixture::new(self.kind, self.label.clone(), index)
    }

    fn fork(&self) {
        self.mutate(|runtime| {
            let (_, source) = runtime
                .observe_fork_source(&BranchId("main".to_owned()))
                .expect("main exposes a fork source");
            runtime
                .fork_branch(BranchId(FEATURE.to_owned()), source)
                .expect("the feature branch forks");
        });
    }

    fn create(&self, branch: &str, key: &str, handle: u64) -> EntityId {
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
        self.mutate(|runtime| {
            let committed = commit(runtime, branch, intent);
            let entity = committed
                .created_entity(&created)
                .expect("the create issued");
            crate::relational_snapshot_release::release_query_snapshot(
                runtime,
                &committed.snapshot,
            );
            entity
        })
    }

    /// Admits the feature-into-main merge under the installed schema, and
    /// executes it when admitted.
    fn merge(&self) -> Result<(), Kind> {
        let unique = self.unique(Some(self.index));
        self.mutate(|runtime| {
            let prepared = prepared_merge(runtime);
            admit_merge_unique_values(unique.fields(), runtime, &prepared)
                .map_err(|denial| denial.kind())?;
            runtime
                .execute_prepared_merge(prepared)
                .expect("an admitted merge executes");
            Ok(())
        })
    }

    fn created_fields(&self, key: &str, label: u64) -> AspectFieldPatch {
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

    fn mutate<T>(&self, mutate: impl FnOnce(&mut RelationalRuntime) -> T) -> T {
        self.handle
            .execute_mutation_with_index_refresh(|runtime| Ok::<_, ()>(mutate(runtime)))
            .expect("the unique field's index refreshes")
            .expect("the fixture mutation is infallible")
    }
}

fn prepared_merge(runtime: &mut RelationalRuntime) -> PreparedMergeExecution {
    let bound = runtime
        .bind_merge_execution_request(merge_request())
        .unwrap();
    runtime.prepare_merge_execution(bound).unwrap()
}

fn merge_request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".to_owned()),
        BranchId(FEATURE.to_owned()),
        MergeIntent::ReconcileIntoTarget,
    )
}

fn commit(
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
        .push_batch(WorkerIntentBatch::new("unique-merge").push(intent))
        .expect("the fixture batch stages");
    transaction.commit(runtime).expect("the fixture commits")
}

fn value(ordinal: u64) -> AspectValue {
    AspectValue::String(InternedString::from(format!("value-{ordinal}")))
}
