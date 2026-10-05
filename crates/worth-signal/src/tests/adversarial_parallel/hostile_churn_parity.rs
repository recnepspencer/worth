use crate::facade::{
    ChangedRegion, CheckedEvaluationContext, EvaluationContext, NodeEvaluationResult, NodeId,
    SignalError,
};
use crate::presentation::harness::{signal_parity_suite, SignalProfileCatalog, SignalScenario};
use crate::tests::leased_execution::support::{request, shared_authority};
use crate::tests::support::{version_ab, ASPECT_A};
use worth_harness::facade::{ComparisonMode, ComparisonProfile, ExecutionRequest};

use super::executor_policy::bounded_contract;

fn branch_output(
    node: NodeId,
    source: NodeId,
    branches: &[NodeId],
    mut read: impl FnMut(NodeId) -> Result<u64, SignalError>,
) -> Result<NodeEvaluationResult, SignalError> {
    let value = if node == source {
        1
    } else if branches.contains(&node) {
        read(source)?
    } else {
        branches.iter().try_fold(0_u64, |sum, &branch| {
            Ok::<_, SignalError>(sum + read(branch)?)
        })?
    };
    let result = NodeEvaluationResult::from_version(version_ab(value, 0));
    Ok(if node == source {
        result
            .with_output_identity("wing-artifact")
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-a"))
    } else {
        result
    })
}

fn scenario(width: usize) -> SignalScenario {
    let mut scenario = SignalScenario::new("adversarial-branch-parity");
    let source = scenario.build_node("source", |graph| {
        graph
            .node()
            .with_contract(bounded_contract(&[]))
            .output_identity()
            .build()
    });
    let branches = (0..width)
        .map(|index| {
            scenario.build_node(format!("branch-{index}"), |graph| {
                graph
                    .node()
                    .with_contract(bounded_contract(&[source]))
                    .partitioned_output()
                    .build()
            })
        })
        .collect::<Vec<_>>();
    scenario.build_node("target", |graph| {
        graph
            .node()
            .with_contract(bounded_contract(&branches))
            .build()
    });
    for index in 0..width {
        scenario
            .partition_detail_dependency(
                &format!("branch-{index}"),
                "source",
                ASPECT_A,
                "wing",
                format!("rib-{index}"),
            )
            .unwrap();
        scenario
            .dependency("target", &format!("branch-{index}"), ASPECT_A)
            .unwrap();
    }
    let serial_branches = branches.clone();
    scenario.set_evaluator(move |ctx: &mut EvaluationContext<'_, ()>| {
        branch_output(ctx.node(), source, &serial_branches, |node| {
            ctx.read_aspect_version(node, ASPECT_A)
                .map(|version| version.get(ASPECT_A))
        })
    });
    scenario.set_checked_evaluator(
        move |ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>| {
            branch_output(ctx.node(), source, &branches, |node| {
                ctx.read(node, ASPECT_A)
            })
        },
    );
    scenario
        .observe("target")
        .with_execution_authority(shared_authority().clone(), request(4, 2_000_000))
}

fn compare_scenario(width: usize, repetitions: usize) {
    let fixture = scenario(width).fixture().unwrap();
    let request = ExecutionRequest::target("observe-target", "target".to_string());
    for _ in 0..repetitions {
        let report = signal_parity_suite(
            fixture.clone(),
            request.clone(),
            SignalProfileCatalog::serial("serial-baseline"),
        )
        .comparison_profile(ComparisonProfile {
            mode: ComparisonMode::Semantic,
            include_extensions: false,
            numeric_tolerance: None,
        })
        .candidates([
            SignalProfileCatalog::staged_parallel("checked-staged"),
            SignalProfileCatalog::full_parallel("checked-full"),
        ])
        .compare()
        .unwrap();
        assert!(report.matched);
    }
}

#[test]
fn harness_parity_holds_for_branchy_partitioned_output_identity_graph() {
    compare_scenario(2, 1);
}

#[test]
#[ignore = "stress coverage for wide-graph leased parity loops"]
fn stress_repeated_parallel_parity_on_wide_branch_graph() {
    compare_scenario(24, 25);
}
