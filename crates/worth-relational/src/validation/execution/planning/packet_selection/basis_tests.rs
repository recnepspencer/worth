//! A receipt says why a custom rule did not run. Only a proven footprint
//! outside the rule's declared closure is vacuous truth; deleting inside the
//! closure still runs the rule.
use crate::facade::{
    identity::{EntityId, KindId, PartitionId},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
};
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, DeleteEntityIntent, EntityMutationIntent, EntitySpec,
    MutationIntent, WorkerIntentBatch,
};
use crate::validation::data::*;
use std::{
    num::{NonZeroU64, NonZeroUsize},
    sync::Arc,
};
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

const UNRELATED: KindId = KindId(1);
const GOVERNED: KindId = KindId(2);

struct ClosureRule;
impl CustomInvariantRule for ClosureRule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("basis.closure"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Closure rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(NonZeroU64::new(1_000_000).unwrap()),
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![GOVERNED],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![GOVERNED],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: false,
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                // Every group, so plan-contract group selection never hides the
                // rule: these courts judge only the closure basis.
                groups: InvariantGroupSet::all(),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn prepare_scope_checked(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), CustomInvariantPreparationError>> {
        Some(Ok(()))
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
    fn evaluate_checked(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(Ok(CustomInvariantVerdict::Pass))
    }
}

fn runtime() -> RelationalRuntime {
    runtime_with_native_rules(vec![])
}

fn runtime_with_native_rules(registrations: Vec<InvariantRegistration>) -> RelationalRuntime {
    let mut schema = RelationalSchemaRegistry::new();
    for (kind, name) in [(UNRELATED, "unrelated"), (GOVERNED, "governed")] {
        schema = schema
            .register_entity_kind(EntityKindRegistration {
                kind_id: kind,
                kind_name: name.into(),
                schema_id: SchemaId("basis".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
            .unwrap();
    }
    RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .invariant_catalog(InvariantCatalog { registrations })
        .custom_invariant(CustomInvariantRegistration::new(ClosureRule).unwrap())
        .build()
}

fn lease() -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(10_000_000)),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}

fn create(kind: KindId, key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: kind,
        client_key: crate::symbols::data::ClientKey::raw(key),
        fields: Default::default(),
    }))
}

fn delete(entity_id: EntityId) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
        entity_id,
    }))
}

fn seed(runtime: &RelationalRuntime, lease: &ExecutionResourceLease<'_>, kind: KindId) -> EntityId {
    let key = format!("seed-{}", kind.0);
    let mut tx = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    let request = ExecutionRequest::leased(lease);
    tx.push_batch(
        WorkerIntentBatch::new("seed").push(create(kind, &key)),
        request.allocation_policy(),
    )
    .unwrap();
    tx.commit(&request, runtime)
        .expect("the closure rule passes its seed")
        .created_entity(&CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: kind,
            client_key: crate::symbols::data::ClientKey::raw(&key),
        })
        .expect("the seed was created")
}

type ReceiptFacts = (
    InvariantDecisionKind,
    Option<CustomInvariantNonApplicability>,
);

/// Validates one candidate and returns every custom receipt's facts.
fn receipts(
    runtime: &RelationalRuntime,
    lease: &ExecutionResourceLease<'_>,
    intents: Vec<MutationIntent>,
) -> Vec<ReceiptFacts> {
    let mut tx = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    let request = ExecutionRequest::leased(lease);
    let batch = intents
        .into_iter()
        .fold(WorkerIntentBatch::new("candidate"), WorkerIntentBatch::push);
    tx.push_batch(batch, request.allocation_policy()).unwrap();
    let validated = tx.validate(&request, runtime).expect("candidate validates");
    validated
        .custom_invariant_execution_receipts()
        .iter()
        .map(|receipt| (receipt.verdict(), receipt.non_applicability()))
        .collect()
}

/// Validates one candidate and returns the closure rule's receipt facts.
fn receipt(
    runtime: &RelationalRuntime,
    lease: &ExecutionResourceLease<'_>,
    intents: Vec<MutationIntent>,
) -> ReceiptFacts {
    let [receipt] = receipts(runtime, lease, intents)[..] else {
        panic!("the closure rule leaves exactly one receipt")
    };
    receipt
}

/// An empty candidate plans no validation, so it carries no receipt, not a
/// vacuous one: a producer requirement reads it as missing and refuses.
#[test]
fn an_empty_candidate_leaves_no_receipt_to_be_vacuous() {
    let (runtime, lease) = (runtime(), lease());
    assert_eq!(receipts(&runtime, &lease, vec![]), []);
}

#[test]
fn a_candidate_outside_the_closure_is_vacuous_and_says_so() {
    let (runtime, lease) = (runtime(), lease());
    assert_eq!(
        receipt(&runtime, &lease, vec![create(UNRELATED, "elsewhere")]),
        (
            InvariantDecisionKind::NotApplicable,
            Some(CustomInvariantNonApplicability::CandidateOutsideClosure)
        )
    );
}

#[test]
fn a_native_pass_retains_the_custom_vacuous_receipt_through_staged_reduction() {
    let runtime = runtime_with_native_rules(vec![InvariantRegistration::for_rule(
        InvariantRule::MaxMergedIntents(1),
        InvariantExecutionPoint::CommitBoundary,
        InvariantFailureEffect::BlockCommit,
    )]);
    let lease = lease();
    let request = ExecutionRequest::leased(&lease);
    let mut tx = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    tx.push_batch(
        WorkerIntentBatch::new("candidate").push(create(UNRELATED, "elsewhere")),
        request.allocation_policy(),
    )
    .unwrap();
    let validated = tx
        .validate(&request, &runtime)
        .expect("candidate validates");
    let native = validated
        .commit_boundary
        .results()
        .iter()
        .filter(|check| {
            matches!(
                check.rule,
                InvariantReportedRule::Native(InvariantRule::MaxMergedIntents(1))
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        native.len(),
        1,
        "the final report retains the actual native check"
    );
    assert_eq!(native[0].verdict, InvariantVerdict::Pass);
    let [custom] = validated.custom_invariant_execution_receipts() else {
        panic!("only the registered custom rule leaves a custom receipt")
    };
    assert_eq!(
        custom.rule_id(),
        &CustomInvariantRuleId::new("basis.closure")
    );
    assert_eq!(
        (custom.verdict(), custom.non_applicability()),
        (
            InvariantDecisionKind::NotApplicable,
            Some(CustomInvariantNonApplicability::CandidateOutsideClosure)
        )
    );
}

#[test]
fn a_candidate_touching_the_closure_runs_the_rule_even_when_it_only_deletes() {
    let (runtime, lease) = (runtime(), lease());
    assert_eq!(
        receipt(&runtime, &lease, vec![create(GOVERNED, "inside")]),
        (InvariantDecisionKind::Passed, None)
    );
    let governed = seed(&runtime, &lease, GOVERNED);
    assert_eq!(
        receipt(&runtime, &lease, vec![delete(governed)]),
        (InvariantDecisionKind::Passed, None)
    );
}

/// A deletion's implicit adjacency keeps kind narrowing from proving the
/// footprint, so the planner's empty scope is reported, never called vacuous.
#[test]
fn a_deletion_outside_the_closure_is_not_proven_vacuous() {
    let (runtime, lease) = (runtime(), lease());
    let unrelated = seed(&runtime, &lease, UNRELATED);
    assert_eq!(
        receipt(&runtime, &lease, vec![delete(unrelated)]),
        (
            InvariantDecisionKind::NotApplicable,
            Some(CustomInvariantNonApplicability::NoApplicableTouch)
        )
    );
}
