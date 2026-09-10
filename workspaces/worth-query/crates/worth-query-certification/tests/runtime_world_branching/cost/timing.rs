use std::time::{Duration, Instant};

use crate::query_probe::{assert_ordinary_work, read};
use crate::world::CourtroomWorld;

const FRESH_REPETITIONS: usize = 3;

pub(crate) fn run_scheduled_public_read_timings() {
    let rustc = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|| "unavailable".to_owned());
    eprintln!(
        "timing_environment os={} arch={} pointer_width={} parallelism={} rustc={rustc}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        usize::BITS,
        std::thread::available_parallelism().map_or(0, usize::from),
    );

    for population in [1, 8, 64] {
        let mut cold = Vec::with_capacity(FRESH_REPETITIONS);
        let mut warm = Vec::with_capacity(FRESH_REPETITIONS);
        for repetition in 0..FRESH_REPETITIONS {
            let world = CourtroomWorld::publish_with_intent_population("blocked", population);
            let root = world.application.current_world();

            let started = Instant::now();
            let cold_read = read(&world, root);
            cold.push(started.elapsed());
            assert_ordinary_work(cold_read.work);

            let started = Instant::now();
            let warm_read = read(&world, root);
            warm.push(started.elapsed());
            assert_ordinary_work(warm_read.work);
            assert_eq!(cold_read.input, warm_read.input);
            eprintln!(
                "timing_sample profile=public_read population={population} repetition={} cold_ns={} warm_ns={}",
                repetition + 1,
                cold.last().unwrap().as_nanos(),
                warm.last().unwrap().as_nanos(),
            );
        }
        report_distribution("cold", population, &cold);
        report_distribution("warm", population, &warm);
    }
}

fn report_distribution(posture: &str, population: usize, samples: &[Duration]) {
    let nanos = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    let mean = nanos.iter().map(|sample| *sample as f64).sum::<f64>() / nanos.len() as f64;
    let variance = nanos
        .iter()
        .map(|sample| {
            let delta = *sample as f64 - mean;
            delta * delta
        })
        .sum::<f64>()
        / nanos.len() as f64;
    let mut ordered = nanos.clone();
    ordered.sort_unstable();
    eprintln!(
        "timing_distribution profile=public_read population={population} posture={posture} repetitions={} variance_ns2={variance:.3} p50_ns={} p95_ns={} p99_ns={}",
        ordered.len(),
        percentile(&ordered, 50),
        percentile(&ordered, 95),
        percentile(&ordered, 99),
    );
}

fn percentile(ordered: &[u128], percentile: usize) -> u128 {
    let rank = percentile
        .saturating_mul(ordered.len())
        .div_ceil(100)
        .saturating_sub(1)
        .min(ordered.len() - 1);
    ordered[rank]
}
