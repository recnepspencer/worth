use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_cad_topology_matrix() {
    let suite = "cad_topology_matrix";

    let assembly_bridge_samples = capture_perf_samples(
        suite,
        "assembly_interface_bridge_wave",
        || {
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            let mut nose = Vec::new();
            let mut tank = Vec::new();
            let mut thrust = Vec::new();
            for index in 0..4 {
                nose.push(create_entity_in_partition(
                    &runtime,
                    &format!("nose-skin-{index}"),
                    PartitionId((index % 2) as u32 + 1),
                ));
                tank.push(create_entity_in_partition(
                    &runtime,
                    &format!("tank-frame-{index}"),
                    PartitionId((index % 2) as u32 + 4),
                ));
                thrust.push(create_entity_in_partition(
                    &runtime,
                    &format!("thrust-mount-{index}"),
                    PartitionId((index % 2) as u32 + 7),
                ));
            }
            for index in 0..3 {
                create_relation_in_partition(
                    &runtime,
                    nose[index],
                    nose[index + 1],
                    &format!("nose-seam-{index}"),
                    PartitionId(30),
                );
                create_relation_in_partition(
                    &runtime,
                    tank[index],
                    tank[index + 1],
                    &format!("tank-bay-{index}"),
                    PartitionId(31),
                );
                create_relation_in_partition(
                    &runtime,
                    thrust[index],
                    thrust[index + 1],
                    &format!("thrust-rib-{index}"),
                    PartitionId(32),
                );
            }
            for index in 0..4 {
                create_relation_in_partition(
                    &runtime,
                    nose[index],
                    tank[index],
                    &format!("nose-to-tank-{index}"),
                    PartitionId(33),
                );
            }

            runtime.performance_access().reset_counters();
            let bridge_started_at = Instant::now();
            let bridge_outcome =
                create_relation_outcome(&runtime, tank[2], thrust[1], "tank-to-thrust-interface");
            let bridge_commit_micros = bridge_started_at.elapsed().as_micros();

            let snapshot = runtime.visibility_authority().snapshot();
            let explicit_targets = vec![
                RecordRef::Entity(nose[1]),
                RecordRef::Entity(nose[2]),
                RecordRef::Entity(tank[1]),
                RecordRef::Entity(tank[2]),
                RecordRef::Entity(thrust[1]),
                RecordRef::Entity(thrust[2]),
            ];
            let explicit_packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "cad-assembly-explicit",
                explicit_targets,
            );
            let explicit_started_at = Instant::now();
            let explicit_outcome = runtime
                .read_truth()
                .execute_query_plan(
                    runtime
                        .read_truth()
                        .plan_query_packet(&snapshot, explicit_packet)
                        .expect("planned cad explicit packet"),
                )
                .expect("cad explicit query outcome");
            let explicit_query_micros = explicit_started_at.elapsed().as_micros();

            let connectivity_started_at = Instant::now();
            let summary = runtime.inspect_what_happened().connectivity_summary(
                &ConnectivityInspectionRequest {
                    scope: InspectionScope::Current,
                    partition_scope: None,
                    relation_kind_scope: Some(vec![KindId(2)]),
                    include_members: false,
                    budget: ConnectivityInspectionBudget {
                        max_entities: 64,
                        max_relations: 64,
                        max_frontier: 64,
                        max_components: 8,
                        max_work_units: 256,
                    },
                },
            );
            let connectivity_summary_micros = connectivity_started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros: bridge_commit_micros
                    + explicit_query_micros
                    + connectivity_summary_micros,
                metrics: perf_metrics!({
                    "bridge_commit_micros": bridge_commit_micros,
                    "explicit_query_micros": explicit_query_micros,
                    "connectivity_summary_micros": connectivity_summary_micros,
                    "bridge_changed_records": bridge_outcome.changed_records.len(),
                    "explicit_query_entities": explicit_outcome.complexity.authoritative_entity_records_emitted,
                    "component_count": summary.component_count,
                    "largest_component_size": summary.largest_component_size,
                    "enumerated_entity_count": summary.enumerated_entity_count,
                    "availability": format!("{:?}", summary.availability),
                    "counters": counters,
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "assembly_interface_bridge_wave",
        &assembly_bridge_samples,
        &[
            ("bridge_commit_micros", &["bridge_commit_micros"]),
            ("explicit_query_micros", &["explicit_query_micros"]),
            (
                "connectivity_summary_micros",
                &["connectivity_summary_micros"],
            ),
            ("explicit_query_entities", &["explicit_query_entities"]),
            ("largest_component_size", &["largest_component_size"]),
        ],
    );
    assert!(assembly_bridge_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &assembly_bridge_samples,
        "cad assembly interface bridges should preserve bounded connectivity and local explicit read surfaces",
        |metrics| {
            metrics["bridge_changed_records"].as_u64() == Some(1)
                && metrics["explicit_query_entities"].as_u64() == Some(6)
                && metrics["component_count"].as_u64() == Some(1)
                && metrics["largest_component_size"].as_u64() == Some(12)
                && metrics["enumerated_entity_count"].as_u64() == Some(12)
                && metrics["availability"].as_str() == Some("Direct")
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "inspection_connectivity_summary_requests") == 1
                && counter_u64(metrics, "query_packet_count") <= 6
                && counter_u64(metrics, "query_scope_unit_count") <= 6
        },
    );
}
