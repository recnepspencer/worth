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
    })
    .unwrap()
}

#[derive(Clone, Copy)]
struct SelectionRule {
    rule_id: &'static str,
    execution_point: InvariantExecutionPoint,
    cost_class: InvariantCostClass,
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
                maximum_work_units: std::num::NonZeroU64::new(4096).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract::default(),
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
