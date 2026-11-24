//! The L6 proposal's private invariant report must retain both staged results.
use crate::facade::runtime::RelationalRuntime;
use crate::facade::runtime::RelationalRuntimeApi;
use crate::identity::data::KindId;
use crate::identity::data::PartitionId;
use crate::schema::data::EntityKindRegistration;
use crate::schema::data::KindAspectContractDeclarations;
use crate::schema::data::RelationalSchemaRegistry;
use crate::schema::data::SchemaId;
use crate::schema::data::SchemaVersionId;
use crate::transactions::data::intents::mutation_intent::CreateIntent;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::primitives::EntitySpec;
use crate::transactions::data::primitives::WorkerIntentBatch;
use crate::validation::{
    custom_rule::{
        CustomInvariantExecutionContext, CustomInvariantLeaseBudget, CustomInvariantRegistration,
        CustomInvariantRule, CustomInvariantScopePlanner,
    },
    data::{
        catalog::{InvariantCatalog, InvariantRegistration},
        custom_rule::errors::{
            CustomInvariantExecutionError, CustomInvariantPreparationError, CustomInvariantVerdict,
        },
        descriptor::{
            CustomInvariantAccessContract, CustomInvariantDescriptor,
            CustomInvariantOperationalMetadata,
        },
        execution::{
            InvariantDecisionKind, InvariantExecutionPoint, InvariantFailureEffect,
            InvariantReportedRule, InvariantVerdict,
        },
        groups::{InvariantCostClass, InvariantGroupSet},
        non_applicability::CustomInvariantNonApplicability,
        rule_id::{
            CustomInvariantRuleId, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
        },
        rules::InvariantRule,
    },
};
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
