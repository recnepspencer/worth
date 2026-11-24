use super::{plan_invariant_execution, planned_proof_boundary_summary};
use crate::identity::data::{KindId, PartitionId};
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::transactions::data::intents::mutation_intent::{CreateIntent, MutationIntent};
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::transactions::data::primitives::{EntityReference, RelationSpec, TransactionId};
use crate::validation::engine::{
    request::test_support::RequestGraph, InvariantPlanScopeClass, InvariantScopeWideningCause,
};

#[test]
fn planner_packets_only_include_relation_integrity_registrations_authorized_by_plan_scope() {
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let graph = RequestGraph::new(2, &[]);
    let plan = planned_relation(&graph);
    let instrumentation = RuntimeInstrumentation::default();
    let view = RequestGraph::view(&instrumentation);
    let request = graph.request(&view, Some(&plan));
    let prepared = plan_invariant_execution(
        &view,
        *execution,
        &request,
        None,
        super::InvariantStage::All,
    );
    let packet_relation_kinds = prepared
        .packets
        .iter()
        .filter_map(|packet| match &packet.registration {
            crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::Native(registration) => {
                relation_kind_for_rule(&registration.rule)
            }
            crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::Custom { .. }
            | crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::CustomNotApplicable { .. } => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(packet_relation_kinds, vec![KindId(2)]);
    assert_eq!(prepared.packets.len(), 1);
}

#[test]
fn planner_proof_boundary_reports_partition_scoped_relation_integrity_packets() {
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let graph = RequestGraph::new(2, &[]);
    let plan = planned_relation(&graph);
    let instrumentation = RuntimeInstrumentation::default();
    let view = RequestGraph::view(&instrumentation);
    let request = graph.request(&view, Some(&plan));
    let prepared = plan_invariant_execution(
        &view,
        *execution,
        &request,
        None,
        super::InvariantStage::All,
    );
    let summary = planned_proof_boundary_summary(&prepared);

    assert_eq!(
        summary.scope_class(),
        InvariantPlanScopeClass::PartitionScope
    );
    assert!(summary.widened_causes().is_empty());
    assert_eq!(summary.packet_count(), 1);
    assert_eq!(summary.touched_partition_count(), 1);
}

#[test]
fn planner_proof_boundary_reports_broader_scope_when_no_merged_plan_is_available() {
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let graph = RequestGraph::new(0, &[]);
    let instrumentation = RuntimeInstrumentation::default();
    let view = RequestGraph::view(&instrumentation);
    let request = graph.request(&view, None);
    let prepared = plan_invariant_execution(
        &view,
        *execution,
        &request,
        None,
        super::InvariantStage::All,
    );
    let summary = planned_proof_boundary_summary(&prepared);

    assert_eq!(summary.scope_class(), InvariantPlanScopeClass::BroaderScope);
    assert_eq!(
        summary.widened_causes(),
        &[InvariantScopeWideningCause::AllObservedPartitionScope]
    );
}

fn relation_kind_for_rule(
    rule: &crate::validation::data::rules::InvariantRule,
) -> Option<crate::identity::data::KindId> {
    match rule {
        crate::validation::data::rules::InvariantRule::EndpointKindContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::validation::data::rules::InvariantRule::CardinalityMaximumContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::validation::data::rules::InvariantRule::CardinalityMinimumContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::validation::data::rules::InvariantRule::UniquenessContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::validation::data::rules::InvariantRule::SymmetryContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::validation::data::rules::InvariantRule::EndpointDeletionIntegrityContract(
            contract,
        ) => Some(contract.relation_kind_id),
        _ => None,
    }
}

fn planned_relation(graph: &RequestGraph) -> MergedCommitPlan {
    MergedCommitPlan {
        transaction_id: TransactionId(1),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("planned"),
                source: EntityReference::Existing(graph.entities[0]),
                target: EntityReference::Existing(graph.entities[1]),
                fields: Default::default(),
            },
        ))],
    }
}
