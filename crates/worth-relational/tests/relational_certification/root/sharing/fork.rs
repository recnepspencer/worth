use super::world::supply_chain::SupplyChainScale;
use super::world::supply_chain::{assert_oracle_matches, certified_supply_chain_world};
use worth_relational::facade::history::BranchId;

#[test]
fn phase5_fork_observation_reports_shared_root_and_distinct_cells() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, expected) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &expected);
    fork_from_main(&world.runtime, "storm");
    fork_from_main(&world.runtime, "maintenance");

    let identities = [
        world.runtime.main_branch_identity(),
        branch_identity(&world.runtime, "storm"),
        branch_identity(&world.runtime, "maintenance"),
    ];
    let observation = world.runtime.observe_branch_sharing(&identities).unwrap();
    assert_eq!(
        observation.inspection_version(),
        worth_relational::facade::inspection::RELATIONAL_SHARING_INSPECTION_VERSION
    );
    assert_eq!(
        observation.byte_metric_scope(),
        worth_relational::facade::inspection::RelationalSharingByteMetricScope::CompleteAuthoritativeOwnerAllocations
    );
    assert_eq!(observation.branch_count(), 3);
    assert_eq!(observation.unique_root_count(), 1);
    assert_eq!(observation.unique_canonical_commit_artifacts(), 1);
    assert_eq!(observation.copied_truth_bytes(), 0);
    assert_eq!(observation.copied_commit_envelopes(), 0);
    assert_eq!(observation.shared_root_acquisitions(), 2);
    assert_eq!(observation.fork_materialized_entity_count(), 0);
    assert_eq!(observation.fork_materialized_relation_count(), 0);
    assert_eq!(observation.coordination_waits(), 0);
    assert_eq!(
        observation.logical_branch_partition_payload_bytes(),
        observation.unique_physical_partition_payload_bytes() * observation.branch_count()
    );
    assert_eq!(observation.coordination_cells().len(), 3);
    assert_eq!(
        observation
            .coordination_cells()
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
}

#[test]
#[ignore = "scheduled maximum-scale fork slope"]
fn phase5_standard_fork_copy_slope_is_flat_through_4096_forks() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let samples = standard_fanout_samples(execution, &[1, 64, 4_096]);
    assert_eq!(
        samples
            .iter()
            .map(|sample| sample.fork_materialized_authoritative_bytes)
            .collect::<Vec<_>>(),
        vec![0; 3],
        "the exact zero-copy counter remains flat at every declared fan-out"
    );
    assert_eq!(
        fitted_slope(&samples),
        0.0,
        "least-squares fork-copy slope over log2 fan-out remains exactly flat"
    );
}

#[derive(Clone, Copy)]
struct ForkFanoutSample {
    log2_fanout: f64,
    fork_materialized_authoritative_bytes: u64,
}

fn standard_fanout_samples(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    fanouts: &[u32],
) -> Vec<ForkFanoutSample> {
    let (world, expected) = certified_supply_chain_world(execution, SupplyChainScale::standard());
    assert_oracle_matches(execution, &world, &expected);
    let mut branches = vec![world.runtime.main_branch_identity()];
    let mut samples = Vec::with_capacity(fanouts.len());
    for ordinal in 0..*fanouts.last().expect("at least one fan-out sample") {
        let name = format!("standard-fanout-{ordinal}");
        fork_from_main(&world.runtime, &name);
        branches.push(branch_identity(&world.runtime, &name));
        let fanout = ordinal + 1;
        if fanouts.contains(&fanout) {
            let observation = world.runtime.observe_branch_sharing(&branches).unwrap();
            assert_standard_fanout_observation(&observation, fanout, branches.len());
            samples.push(ForkFanoutSample {
                log2_fanout: f64::from(fanout).log2(),
                fork_materialized_authoritative_bytes: observation
                    .fork_materialized_authoritative_bytes(),
            });
        }
    }
    samples
}

fn assert_standard_fanout_observation(
    observation: &worth_relational::facade::inspection::RelationalBranchSharingObservation,
    fanout: u32,
    observed_branch_count: usize,
) {
    assert_eq!(observation.branch_count(), u64::from(fanout) + 1);
    assert_eq!(observation.unique_root_count(), 1);
    assert_eq!(observation.unique_canonical_commit_artifacts(), 1);
    assert_eq!(observation.shared_root_acquisitions(), u64::from(fanout));
    assert_eq!(observation.copied_truth_bytes(), 0);
    assert_eq!(observation.copied_commit_envelopes(), 0);
    assert_eq!(observation.fork_materialized_entity_count(), 0);
    assert_eq!(observation.fork_materialized_relation_count(), 0);
    assert_eq!(observation.fork_materialized_authoritative_bytes(), 0);
    assert_eq!(
        observation.coordination_cells().len(),
        observed_branch_count
    );
    assert_eq!(
        observation.logical_branch_partition_payload_bytes(),
        observation.unique_physical_partition_payload_bytes() * observation.branch_count()
    );
    let eager_fork_clone_mutant = observation
        .unique_physical_partition_payload_bytes()
        .saturating_mul(u64::from(fanout));
    assert_ne!(
        observation.fork_materialized_authoritative_bytes(),
        eager_fork_clone_mutant,
        "an eager per-fork truth clone must turn the zero-copy oracle red"
    );
}

fn fitted_slope(samples: &[ForkFanoutSample]) -> f64 {
    let sample_count = samples.len() as f64;
    let mean_x = samples.iter().map(|sample| sample.log2_fanout).sum::<f64>() / sample_count;
    let mean_y = samples
        .iter()
        .map(|sample| sample.fork_materialized_authoritative_bytes as f64)
        .sum::<f64>()
        / sample_count;
    let covariance = samples
        .iter()
        .map(|sample| {
            (sample.log2_fanout - mean_x)
                * (sample.fork_materialized_authoritative_bytes as f64 - mean_y)
        })
        .sum::<f64>();
    let variance = samples
        .iter()
        .map(|sample| (sample.log2_fanout - mean_x).powi(2))
        .sum::<f64>();
    covariance / variance
}

fn fork_from_main(runtime: &worth_relational::facade::runtime::RelationalRuntime, name: &str) {
    let (_, source_basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    runtime
        .fork_branch(BranchId(name.to_owned()), source_basis)
        .expect("fork remains metadata-only");
}

fn branch_identity(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    name: &str,
) -> worth_relational::facade::branch::RelationalBranchIdentity {
    runtime
        .branch_identity(&BranchId(name.to_owned()))
        .expect("branch identity is owner-issued")
}
