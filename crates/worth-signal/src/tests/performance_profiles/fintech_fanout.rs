use std::sync::Once;
use std::time::Instant;

use serde_json::{json, Value};

use crate::tests::domains::fintech::{setup_seeded_world_with, FintechScale, MarketRegime};
use crate::tests::performance_support::{
    capture_and_certify_perf_samples, PerfMeasurement, PerfTimingPolicy,
};

use super::{eval_metrics_delta, perf_contract, policy_for};

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture"]
fn perf_fintech_mixed_fanout_profile_matrix() {
    for profile_name in ["operational", "development", "forensic"] {
        match profile_name {
            "operational" => {
                static OPERATIONAL_WARMUP: Once = Once::new();
                OPERATIONAL_WARMUP.call_once(|| {
                    let mut world =
                        setup_seeded_world_with(FintechScale::fanout(), MarketRegime::Calm, 7);
                    world.set_runtime_policy(policy_for("operational"));
                    let _ = world.read_top_desk();
                    let _ = world.read_top_scenario();
                    let _ = world.bump_primary_market(7, 4, 2, 1);
                    let _ = world.read_top_desk();
                    let _ = world.read_top_scenario();
                });
            }
            "development" => {
                static DEVELOPMENT_WARMUP: Once = Once::new();
                DEVELOPMENT_WARMUP.call_once(|| {
                    let mut world =
                        setup_seeded_world_with(FintechScale::fanout(), MarketRegime::Calm, 7);
                    world.set_runtime_policy(policy_for("development"));
                    let _ = world.read_top_desk();
                    let _ = world.read_top_scenario();
                    let _ = world.bump_primary_market(7, 4, 2, 1);
                    let _ = world.read_top_desk();
                    let _ = world.read_top_scenario();
                });
            }
            "forensic" => {
                static FORENSIC_WARMUP: Once = Once::new();
                FORENSIC_WARMUP.call_once(|| {
                    let mut world =
                        setup_seeded_world_with(FintechScale::fanout(), MarketRegime::Calm, 7);
                    world.set_runtime_policy(policy_for("forensic"));
                    let _ = world.read_top_desk();
                    let _ = world.read_top_scenario();
                    let _ = world.bump_primary_market(7, 4, 2, 1);
                    let _ = world.read_top_desk();
                    let _ = world.read_top_scenario();
                });
            }
            other => panic!("unexpected profile for perf test: {other}"),
        }

        let samples = capture_and_certify_perf_samples(
            perf_contract(
                "fintech_mixed_fanout",
                profile_name,
                match profile_name {
                    "operational" => PerfTimingPolicy::StrictHeavy,
                    "development" | "forensic" => PerfTimingPolicy::MedianOnly,
                    other => panic!("unexpected profile for perf test: {other}"),
                },
                &["read_before_nanos", "mutation_nanos", "read_after_nanos"],
            ),
            || {
                let mut world =
                    setup_seeded_world_with(FintechScale::fanout(), MarketRegime::Calm, 7);
                world.set_runtime_policy(policy_for(profile_name));

                let warmup_start = Instant::now();
                let _ = world.read_top_desk().unwrap();
                let _ = world.read_top_scenario().unwrap();
                let warmup_nanos = warmup_start.elapsed().as_nanos();

                let before = world.runtime_metrics();
                let read_before_start = Instant::now();
                let _ = world.read_top_desk().unwrap();
                let _ = world.read_top_scenario().unwrap();
                let read_before_nanos = read_before_start.elapsed().as_nanos();

                let mutation_start = Instant::now();
                let _ = world.bump_primary_market(7, 4, 2, 1).unwrap();
                let mutation_nanos = mutation_start.elapsed().as_nanos();

                let read_after_start = Instant::now();
                let _ = world.read_top_desk().unwrap();
                let _ = world.read_top_scenario().unwrap();
                let read_after_nanos = read_after_start.elapsed().as_nanos();
                let after = world.runtime_metrics();

                assert!(after.evaluation.evaluation_calls >= before.evaluation.evaluation_calls);
                let mut metrics = eval_metrics_delta(before, after);
                if let Value::Object(ref mut map) = metrics {
                    map.insert("warmup_nanos".into(), json!(warmup_nanos));
                    map.insert("read_before_nanos".into(), json!(read_before_nanos));
                    map.insert("mutation_nanos".into(), json!(mutation_nanos));
                    map.insert("read_after_nanos".into(), json!(read_after_nanos));
                }
                PerfMeasurement::new(
                    (read_before_nanos + mutation_nanos + read_after_nanos) / 1_000,
                    metrics,
                )
            },
        );

        assert!(samples.iter().all(|sample| sample.elapsed_micros > 0));
    }
}
