use std::sync::Arc;

use crate::facade::identity::PartitionId;
use crate::facade::runtime::RelationalRuntimeApi;
use crate::identity::data::KindId;
use crate::symbols::data::ClientKey;
use crate::transactions::data::{
    CreateIntent, EntitySpec, MergedCommitPlan, MutationIntent, TransactionId,
};
use crate::validation::data::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
    CustomInvariantScopePlanner, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
    CustomInvariantVerdict, InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantGroup, InvariantGroupSet, InvariantReportedRule, InvariantVerdict,
};
use crate::validation::invariant_access::test_support::{
    evaluate_main_commit_boundary_plan, evaluate_main_graph_composition_plan,
};

#[test]
fn graph_composition_plan_selects_only_graph_composition_custom_registrations() {
    let runtime = RelationalRuntimeApi::builder()
        .custom_invariant(registration(
            "shared.rule",
            InvariantExecutionPoint::GraphComposition,
            InvariantCostClass::Touched,
        ))
        .custom_invariant(registration(
            "shared.rule",
            InvariantExecutionPoint::CommitBoundary,
            InvariantCostClass::Touched,
        ))
        .custom_invariant(registration(
            "commit.only",
            InvariantExecutionPoint::CommitBoundary,
            InvariantCostClass::Touched,
        ))
        .custom_invariant(registration(
            "graph.only",
            InvariantExecutionPoint::GraphComposition,
            InvariantCostClass::Touched,
        ))
        .build();
    let plan = graph_relevant_plan(71);

    let graph_result = evaluate_main_graph_composition_plan(&runtime, &plan);
    let commit_result = evaluate_main_commit_boundary_plan(&runtime, &plan);

    assert_custom_rule_ids(
        &graph_result,
        InvariantExecutionPoint::GraphComposition,
        &["graph.only", "shared.rule"],
    );
    assert_custom_rule_ids(
        &commit_result,
        InvariantExecutionPoint::CommitBoundary,
        &["commit.only", "shared.rule"],
    );
}

#[test]
fn graph_composition_plan_does_not_execute_global_cost_custom_registration() {
    let runtime = RelationalRuntimeApi::builder()
        .custom_invariant(registration(
            "graph.global",
            InvariantExecutionPoint::GraphComposition,
            InvariantCostClass::Global,
        ))
        .build();
    runtime.performance_access().reset_counters();

    let result = evaluate_main_graph_composition_plan(&runtime, &graph_relevant_plan(72));

    assert_eq!(
        result.metadata().execution_point(),
        InvariantExecutionPoint::GraphComposition
    );
    assert_eq!(result.metadata().max_cost(), InvariantCostClass::Touched);
    assert!(result.results().is_empty());
    assert_eq!(
        runtime
            .performance_access()
            .counters()
            .custom_invariant_execution_count,
        0
    );
}

#[test]
fn custom_registration_runs_only_for_its_declared_affected_kind() {
    let runtime = RelationalRuntimeApi::builder()
        .custom_invariant(scoped_registration("matching.rule", KindId(1)))
        .custom_invariant(scoped_registration("unrelated.rule", KindId(2)))
        .build();

    let result = evaluate_main_commit_boundary_plan(&runtime, &graph_relevant_plan(73));

    assert_custom_rule_ids(
        &result,
        InvariantExecutionPoint::CommitBoundary,
        &["matching.rule"],
    );
}

#[test]
fn unrelated_large_candidate_does_not_spend_a_custom_rules_work_budget() {
    let runtime = RelationalRuntimeApi::builder()
        .custom_invariant(scoped_registration_with_work(
            "unrelated.rule",
            KindId(2),
            16,
        ))
        .build();
    let plan = large_kind_one_plan(74);
    let result = evaluate_main_commit_boundary_plan(&runtime, &plan);
    assert!(result.results().iter().any(|check| {
        matches!(check.rule, InvariantReportedRule::Custom(ref identity) if identity.rule_id.as_str() == "unrelated.rule")
            && matches!(check.verdict, InvariantVerdict::NotApplicable)
    }));
}

#[test]
fn related_large_candidate_still_spends_its_custom_rules_work_budget() {
    let runtime = RelationalRuntimeApi::builder()
        .custom_invariant(scoped_registration_with_work("related.rule", KindId(1), 16))
        .build();
    let result = evaluate_main_commit_boundary_plan(&runtime, &large_kind_one_plan(75));
    assert!(result.results().iter().any(|check| {
        matches!(check.rule, InvariantReportedRule::Custom(ref identity) if identity.rule_id.as_str() == "related.rule")
            && matches!(check.verdict, InvariantVerdict::Violation(_))
    }));
}

fn large_kind_one_plan(transaction: u64) -> MergedCommitPlan {
    let mut plan = graph_relevant_plan(transaction);
    for index in 0..512 {
        plan.merged_intents
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(format!("unrelated-{index}")),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            })));
    }
    plan
}

fn assert_custom_rule_ids(
    result: &crate::validation::engine::InvariantExecutionResult,
    execution_point: InvariantExecutionPoint,
    expected_rule_ids: &[&str],
) {
    let mut actual_rule_ids = result
        .results()
        .iter()
        .filter(|check| check.execution_point == execution_point)
        .filter(|check| matches!(check.verdict, InvariantVerdict::Violation(_)))
        .filter_map(|check| match &check.rule {
            InvariantReportedRule::Custom(identity) => Some(identity.rule_id.as_str().to_string()),
            InvariantReportedRule::Native(_) => None,
        })
        .collect::<Vec<_>>();
    actual_rule_ids.sort();

    assert_eq!(
        actual_rule_ids,
        expected_rule_ids
            .iter()
            .map(|rule_id| rule_id.to_string())
            .collect::<Vec<_>>()
    );
}

fn graph_relevant_plan(transaction_id: u64) -> MergedCommitPlan {
    MergedCommitPlan {
        transaction_id: TransactionId(transaction_id),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(format!("graph-selection-{transaction_id}")),
            fields: crate::transactions::data::AspectFieldPatch::default(),
        }))],
    }
}

fn registration(
    rule_id: &'static str,
    execution_point: InvariantExecutionPoint,
    cost_class: InvariantCostClass,
) -> CustomInvariantRegistration {
    CustomInvariantRegistration::new(SelectionRule {
        rule_id,
        execution_point,
        cost_class,
        affected_entity_kind: None,
        maximum_work_units: 4096,
    })
    .unwrap()
}

fn scoped_registration(rule_id: &'static str, kind: KindId) -> CustomInvariantRegistration {
    scoped_registration_with_work(rule_id, kind, 4096)
}

fn scoped_registration_with_work(
    rule_id: &'static str,
    kind: KindId,
    maximum_work_units: u64,
) -> CustomInvariantRegistration {
    CustomInvariantRegistration::new(SelectionRule {
        rule_id,
        execution_point: InvariantExecutionPoint::CommitBoundary,
        cost_class: InvariantCostClass::Touched,
        affected_entity_kind: Some(kind),
        maximum_work_units,
    })
    .unwrap()
}

#[derive(Clone, Copy)]
struct SelectionRule {
    rule_id: &'static str,
    execution_point: InvariantExecutionPoint,
    cost_class: InvariantCostClass,
    affected_entity_kind: Option<KindId>,
    maximum_work_units: u64,
}

impl CustomInvariantRule for SelectionRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(self.rule_id),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from(self.rule_id),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(self.maximum_work_units).unwrap(),
                access: self.affected_entity_kind.map_or_else(
                    crate::validation::data::CustomInvariantAccessContract::default,
                    |kind| crate::validation::data::CustomInvariantAccessContract {
                        read_entity_kinds: vec![kind],
                        read_relation_kinds: Vec::new(),
                        affected_entity_kinds: vec![kind],
                        affected_relation_kinds: Vec::new(),
                    },
                ),
                execution_point: self.execution_point,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: self.cost_class,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Violation)
    }
}
