//! Role-aware selected-node copy admission before an evaluator can run.
use super::{overflow, ResourceAdmission};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

pub(super) fn clone_heap(
    admission: &ResourceAdmission<'_, '_>,
    graph: &SignalGraph,
    producer: NodeId,
    selected: &[NodeId],
    work: &mut Work,
) -> Result<u64, SignalError> {
    let mut bytes = 0_u64;
    for &node in selected {
        let additional = if node == producer && !admission.producers.contains(&node) {
            // A selected consumer promoted to producer receives a full draft.
            // The prior operational clone remains charged in this epoch.
            graph.epoch_producer_draft_clone_charge(node, work)?.bytes()
        } else if !admission.selected.contains(&node) {
            graph.epoch_consumer_clone_charge(node, work)?.bytes()
        } else {
            0
        };
        bytes = bytes.checked_add(additional).ok_or_else(overflow)?;
    }
    Ok(bytes)
}

pub(super) fn root_growth(
    admission: &ResourceAdmission<'_, '_>,
    graph: &SignalGraph,
    producer: NodeId,
    selected: &[NodeId],
    work: &mut Work,
    budget: &mut SignalPreparationBudget,
) -> Result<u64, SignalError> {
    graph.epoch_selected_node_root_growth_bound(
        &admission.selected,
        selected,
        &admission.producers,
        producer,
        work,
        budget,
    )
}

impl ResourceAdmission<'_, '_> {
    pub(super) fn accept_selected(
        &mut self,
        nodes: Vec<NodeId>,
        producer: NodeId,
        consumers: &[NodeId],
    ) {
        self.selected.extend(nodes);
        self.producers.insert(producer);
        self.consumers.extend(consumers.iter().copied());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::aspect::{Aspect, AspectVersion};
    use crate::data::comparator::DefaultComparatorPolicyResolver;
    use crate::data::node::{BoundedSignalInputs, DeclaredSignalInput, NodeContract};
    use crate::data::request_preparation::SignalPreparationBudget;
    use crate::data::retained_storage::RetainedStoragePreparation;
    use crate::logic::evaluation::EvaluationRequestMode;
    use crate::logic::planner::{EligibleTask, EligibleTaskAdmission, TaskReason};
    use crate::tests::leased_execution::support::{authority, request};
    use worth_execution::{ExecutionScan, MapKernelFailure, ScanOutcome};
    use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy, PartitionIdentity};

    const VALUE: Aspect = Aspect::new(0);

    fn funded_request() -> worth_execution::LeaseRequest {
        let mut admission = request(4, 20_000_000);
        admission.policy = ExecutionRequestPolicy::new(
            admission.policy.posture(),
            admission.policy.determinism(),
            ExecutionBudget::new(
                admission.policy.budget().max_workers(),
                128 * 1024 * 1024,
                20_000_000,
            ),
        );
        admission
    }

    fn task(node: NodeId) -> EligibleTask {
        EligibleTask {
            node,
            request_mode: EvaluationRequestMode::Default,
            direct_request: true,
            reason: TaskReason::RequestedTarget,
            admission: EligibleTaskAdmission::default(),
        }
    }

    fn probe(
        graph: &mut SignalGraph,
        source: NodeId,
        consumer: NodeId,
        lease: &worth_execution::ExecutionResourceLease<'_>,
        capacity: u64,
    ) -> (u64, u64, bool) {
        let identity = PartitionIdentity::new(1);
        let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).unwrap();
        let mut evidence = None;
        let outcome = scan.run(
            Some(lease),
            (),
            0,
            0,
            1024 * 1024,
            1024 * 1024,
            |_, _, request_work| {
                let mut budget = SignalPreparationBudget::new(capacity);
                graph
                    .prepare_epoch_topology_storage_readiness(
                        Some(&mut *request_work),
                        Some(&mut budget),
                    )
                    .unwrap();
                let mut admission = ResourceAdmission::new(&budget, lease).unwrap();
                let comparator = DefaultComparatorPolicyResolver::default();
                assert!(admission
                    .consider(
                        graph,
                        &task(source),
                        &comparator,
                        Some(&mut *request_work),
                        &mut budget
                    )
                    .unwrap());
                assert!(admission.selected.contains(&consumer));
                assert!(!admission.producers.contains(&consumer));
                let first = admission.fixed;
                let accepted = admission
                    .consider(
                        graph,
                        &task(consumer),
                        &comparator,
                        Some(&mut *request_work),
                        &mut budget,
                    )
                    .unwrap();
                evidence = Some((first, admission.fixed, accepted));
                Ok::<_, MapKernelFailure<SignalError>>(((), ()))
            },
        );
        assert!(matches!(outcome, ScanOutcome::Complete { .. }));
        evidence.expect("one resource probe ran")
    }

    #[test]
    fn selected_consumer_upgrade_requires_full_producer_capacity_before_edit() {
        use crate::data::output::NodeEvaluationResult;
        let mut graph = SignalGraph::new();
        let source = graph
            .node()
            .with_contract(
                NodeContract::wildcard()
                    .with_produces(VALUE)
                    .with_bounded_inputs(BoundedSignalInputs::default()),
            )
            .build();
        let consumer = graph
            .node()
            .with_contract(
                NodeContract::wildcard()
                    .with_produces(VALUE)
                    .with_bounded_inputs(BoundedSignalInputs::new([DeclaredSignalInput::new(
                        source, VALUE,
                    )])),
            )
            .build();
        let setup = authority().request_lease(funded_request()).unwrap();
        graph
            .evaluate_checked(
                &[consumer],
                EvaluationRequestMode::Default,
                &(),
                &|ctx| {
                    let result =
                        NodeEvaluationResult::from_version(AspectVersion::zero().with(VALUE, 1));
                    Ok(if ctx.node() == consumer {
                        let _ = ctx.read(source, VALUE)?;
                        result.with_label("cold-consumer-evidence".repeat(4_096))
                    } else {
                        result
                    })
                },
                &setup,
            )
            .unwrap();
        drop(setup);
        assert_eq!(graph.subscribers_of(source).unwrap(), &[consumer]);
        let mut measurement = RetainedStoragePreparation::new(usize::MAX);
        let full = graph
            .epoch_node_clone_charge(consumer, &mut measurement)
            .unwrap()
            .bytes();
        let operational = graph
            .epoch_consumer_clone_charge(consumer, &mut measurement)
            .unwrap()
            .bytes();
        assert!(full > operational + 64 * 1024);
        let lease = authority().request_lease(funded_request()).unwrap();
        let (first, final_fixed, accepted) =
            probe(&mut graph, source, consumer, &lease, 16 * 1024 * 1024);
        assert!(accepted);
        assert!(final_fixed >= first + full);
        let (short_first, short_final, short_accepted) =
            probe(&mut graph, source, consumer, &lease, final_fixed - 1);
        assert!(!short_accepted);
        assert_eq!(short_final, short_first);
        assert_eq!(graph.node_aspect_version(consumer).unwrap().get(VALUE), 1);
    }
}
