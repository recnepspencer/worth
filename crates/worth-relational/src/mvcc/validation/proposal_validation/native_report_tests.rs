//! The L6 proposal's private invariant report must retain both staged results.
use crate::facade::runtime::RelationalRuntime;
use crate::facade::runtime::RelationalRuntimeApi;
use crate::identity::data::KindId;
use crate::identity::data::PartitionId;
use crate::transactions::data::intents::mutation_intent::CreateIntent;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::primitives::EntitySpec;
use crate::transactions::data::primitives::WorkerIntentBatch;
use crate::validation::{
    custom_rule::CustomInvariantRegistration,
    data::{
        catalog::{InvariantCatalog, InvariantRegistration},
        execution::{
            InvariantDecisionKind, InvariantExecutionPoint, InvariantFailureEffect,
            InvariantReportedRule, InvariantVerdict,
        },
        non_applicability::CustomInvariantNonApplicability,
        rule_id::CustomInvariantRuleId,
        rules::InvariantRule,
    },
};
use std::num::NonZeroUsize;
use worth_relational_contracts::internal_port::validation::data::catalog::InvariantRegistrationOwnerPort as _;
#[path = "native_report_fixture.rs"]
mod fixture;
use fixture::UNRELATED;
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn runtime_with_native_rules(registrations: Vec<InvariantRegistration>) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(fixture::schema())
        .invariant_catalog(InvariantCatalog { registrations })
        .custom_invariant(CustomInvariantRegistration::new(fixture::ClosureRule).unwrap())
        .build()
        .expect("valid native runtime configuration")
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
        .state
        .invariants
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
