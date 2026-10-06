//! A primary-graph merge obeys the unique law. The backend that owns the
//! runtime looks up every unique value the merge writes; a runtime lent
//! through the public mutation path never executes a merge, because only
//! the owner knows the graph's application schema.

use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_query_execution::facade::integration::{
    retain_primary_graph_integration_handle, WorthQueryPrimaryGraphIntegrationHandle,
};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};

use crate::basis_lifecycle::basis_lifecycle;
use crate::effect_lifecycle::{
    admit_effect_intent, evaluate_effect_eligibility, normalize_raw_effect_intent,
    scope_admitted_effect_plan, EffectEligibilityOutcome, EffectExecutionAuthority,
    EffectExecutionDenialKind, EffectExecutionStop, RawEffectIntent,
};
use crate::harness::fixtures::effect_authorities::branch_snapshot_identity;
use crate::ordinary::workflow::{branch_merge, declare_branch_merge};
use crate::workflow::{
    synthetic_runtime_workflow_binding_scoped_for_branch_snapshot_binding_identity,
    MergeLoweringInput, WorkflowAuthorityTargetFamily, WorkflowBindingScopeField,
    WorkflowBudgetClass, WorkflowCostClass, WorkflowDeclarationFamily, WorkflowDeclarationRequest,
    WorkflowFreshnessPolicy,
};

use super::primary_graph_merge_runtime;

/// The installation seeds a principal whose unique identity is 11.
const SEEDED_IDENTITY: u64 = 11;

#[test]
fn a_merge_through_the_lent_primary_graph_runtime_is_denied() {
    let world = DuplicateWorld::new();
    let main_before = main_head(&world.graph);

    let mut denial = None;
    world.lend(|relational| {
        denial = Some(
            lowered_merge(relational)
                .execute_receipt_with(EffectExecutionAuthority::relational(relational))
                .map(|_| ()),
        );
    });

    match denial.expect("the lent merge ran") {
        Err(EffectExecutionStop::Denied(denial)) => assert_eq!(
            denial.denial_kind(),
            EffectExecutionDenialKind::MergeRequiresRuntimeOwner
        ),
        other => panic!("a lent merge of a duplicate unique value must be denied: {other:?}"),
    }
    assert_eq!(
        main_head(&world.graph),
        main_before,
        "the duplicate never reaches main"
    );
}

#[test]
fn a_merge_through_the_owning_backend_is_denied_its_duplicate_value() {
    let DuplicateWorld { runtime, graph } = DuplicateWorld::new();
    let main_before = main_head(&graph);
    let mut workspace = runtime
        .workspace("primary-graph-owner-merge")
        .expect("primary-graph workspace opens");
    let declaration = declare_branch_merge("main", "candidate").expect("merge declares");
    let context = branch_merge(&workspace, &declaration).expect("merge authority admits");
    let outcome = declaration.using(context).run(&mut workspace);

    let stop = outcome
        .stop()
        .expect("the owner refuses a merge of a duplicate unique value");
    assert_eq!(
        stop.effect_kind(),
        Some(EffectExecutionDenialKind::UniqueValueTaken),
        "{}",
        stop.message()
    );
    assert_eq!(
        main_head(&graph),
        main_before,
        "the duplicate never reaches main"
    );
}

/// An installed primary graph whose `candidate` branch, forked from main,
/// holds a raw second principal with the seeded identity.
struct DuplicateWorld {
    runtime: crate::runtime::WorthQueryRuntime,
    graph: WorthQueryPrimaryGraphIntegrationHandle,
}

impl DuplicateWorld {
    fn new() -> Self {
        let runtime = primary_graph_merge_runtime();
        let graph = retain_primary_graph_integration_handle(&runtime.execution_runtime)
            .expect("installed runtime retains its primary graph");
        let world = Self { runtime, graph };
        world.lend(|relational| {
            let (_, basis) = relational
                .observe_fork_source(&BranchId("main".to_owned()))
                .expect("main exposes a fork source");
            relational
                .fork_branch(BranchId("candidate".to_owned()), basis)
                .expect("the candidate branch forks");
        });
        world.lend(|relational| create_duplicate_principal(relational, "candidate"));
        world
    }

    fn lend(&self, mutate: impl FnOnce(&mut RelationalRuntime)) {
        self.graph
            .execute_mutation_with_index_refresh(|relational| {
                mutate(relational);
                Ok::<_, ()>(())
            })
            .expect("the lent runtime refreshes its indexes")
            .expect("the fixture mutation is infallible");
    }
}

fn main_head(graph: &WorthQueryPrimaryGraphIntegrationHandle) -> Option<CommitId> {
    graph.with_runtime(|relational| {
        let identity = relational
            .branch_identity(&BranchId("main".to_owned()))
            .ok()?;
        let (_, basis) = relational.observe_branch(&identity).ok()?;
        basis.observation().commit_id()
    })
}

/// A raw create of a second principal holding the seeded identity, which a
/// program write would be refused.
fn create_duplicate_principal(relational: &mut RelationalRuntime, branch: &str) {
    let kind = principal_kind(relational);
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("PrincipalIdentity").expect("aspect key admits"),
        CanonicalFieldPath::single(FieldKey::new("PrincipalIdentityField").expect("field admits")),
    );
    let identity = relational
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("the branch exists");
    let basis = relational
        .admit_branch_basis(&identity)
        .expect("the branch admits");
    let mut transaction = relational
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("the fixture transaction begins");
    transaction
        .push_batch(
            WorkerIntentBatch::new("duplicate-principal").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: kind,
                    client_key: ClientKey::raw("duplicate-principal"),
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        locator,
                        AspectValue::UInt64(SEEDED_IDENTITY),
                    )])),
                }),
            )),
        )
        .expect("the fixture batch stages");
    let committed = transaction
        .commit(relational)
        .expect("the raw duplicate commits");
    relational
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .expect("the fixture releases its commit snapshot");
}

fn principal_kind(relational: &RelationalRuntime) -> KindId {
    relational
        .config()
        .schema
        .registry
        .authority_snapshot()
        .entity_kinds
        .iter()
        .find(|kind| kind.kind_name == "Principal")
        .map(|kind| kind.kind_id)
        .expect("the installed schema registers Principal")
}

/// The candidate-into-main merge, lowered as a public caller lowers it.
fn lowered_merge(
    relational: &RelationalRuntime,
) -> crate::effect_lifecycle::LoweredEffectExecutionPlan {
    let declaration = declare_branch_merge("main", "candidate").expect("merge declares");
    let scope = WorkflowBindingScopeField::Identity(declaration.identity().evidence_identity());
    let binding = synthetic_runtime_workflow_binding_scoped_for_branch_snapshot_binding_identity(
        "lent-primary-graph-merge",
        &scope,
        branch_snapshot_identity(relational, "main"),
        BranchId("main".to_owned()),
    );
    let basis = basis_lifecycle()
        .branch_head("main", true)
        .prepare_mutation()
        .expect("merge basis admits");
    let normalized = normalize_raw_effect_intent(
        &basis.into(),
        RawEffectIntent::Merge {
            binding,
            request: WorkflowDeclarationRequest::new(
                WorkflowDeclarationFamily::MergeLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMerge,
                WorkflowCostClass::MergeLoweringNarrow,
                WorkflowBudgetClass::AuthorityTargetBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            input: MergeLoweringInput::reconcile_into_target(
                BranchId("main".to_owned()),
                BranchId("candidate".to_owned()),
            ),
        },
    )
    .expect("merge normalizes");
    let eligibility = match evaluate_effect_eligibility(normalized) {
        EffectEligibilityOutcome::Admitted(eligibility) => eligibility,
        other => panic!("merge admits, got {other:?}"),
    };
    scope_admitted_effect_plan(admit_effect_intent(eligibility))
        .lower()
        .expect("merge lowers")
}
