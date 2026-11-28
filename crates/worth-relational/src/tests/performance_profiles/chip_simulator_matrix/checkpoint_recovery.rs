use super::*;

pub(super) fn certify_checkpoint_recovery_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let checkpoint_recovery_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::ChipSimulation, &medium);
        let source =
            create_entity_in_partition(execution, &runtime, "persisted-driver", PartitionId(7));
        let targets = (0..12)
            .map(|index| {
                let partition_id = match index % 3 {
                    0 => PartitionId(11),
                    1 => PartitionId(13),
                    _ => PartitionId(17),
                };
                create_entity_in_partition(
                    execution,
                    &runtime,
                    &format!("persisted-sink-{index}"),
                    partition_id,
                )
            })
            .collect::<Vec<_>>();
        for (index, target) in targets.iter().enumerate() {
            create_relation_in_partition(
                execution,
                &runtime,
                source,
                *target,
                &format!("persisted-edge-{index}"),
                PartitionId(29),
            );
        }

        let (_, checkpoint_work) = measure_counted_work(execution, || {
            capture_and_file_checkpoint(execution, &runtime, &medium)
        });
        assert!(medium.has_filed_checkpoint());
        assert_eq!(medium.committed_count(), 25);

        let mut recovered =
            durable_runtime_profile_for_medium(RelationalRuntimeProfile::ChipSimulation, &medium);
        let (recovery_outcome, recover_work) = measure_counted_work(execution, || {
            recover_runtime_from_medium(execution, &mut recovered, &medium)
        });

        let recovered_commit = recovered
            .history()
            .latest_commit()
            .expect("recovered chip commit")
            .clone();

        let (outgoing_relations, adjacency_work) = measure_counted_work(execution, || {
            recovered
                .storage_access()
                .outgoing_relations_for_entity(execution, source, recovered_commit.version_id)
                .expect("funded adjacency read succeeds")
        });

        CountedPerfSample {
            charged_work: checkpoint_work + recover_work + adjacency_work,
            metrics: perf_metrics!({
                "checkpoint_work": checkpoint_work,
                "recover_work": recover_work,
                "adjacency_work": adjacency_work,
                "recovered_commits": recovery_outcome.recovered_commits,
                "recovered_durable_log_len": recovered.durability().durable_log().len(),
                "outgoing_relation_count": outgoing_relations.len(),
                "counters": recovered.performance_access().counters(),
            }),
        }
    });
    assert!(checkpoint_recovery_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &checkpoint_recovery_samples,
        "chip checkpoint window recovery should preserve fanout truth after recovery",
        |metrics| {
            metrics["recovered_commits"].as_u64().unwrap_or(0) >= 1
                && metrics["outgoing_relation_count"].as_u64() == Some(12)
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );
}
