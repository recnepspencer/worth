use super::*;

fn distribution(profile: &str, axis: &str, posture: &str, samples: &mut [u128]) {
    samples.sort_unstable();
    let mean = samples.iter().sum::<u128>() as f64 / samples.len() as f64;
    let variance = samples
        .iter()
        .map(|n| (*n as f64 - mean).powi(2))
        .sum::<f64>()
        / samples.len() as f64;
    let percentile = |percent: usize| samples[((samples.len() - 1) * percent).div_ceil(100)];
    println!("profile={profile} axis={axis} posture={posture} n={} variance_ns2={variance:.1} p50_ns={} p95_ns={} p99_ns={}", samples.len(), percentile(50), percentile(95), percentile(99));
}
fn profile(name: &str, sizes: [usize; 3], repetitions: usize, warm: usize) {
    println!("profile={name} config=branches128/history512/observations512/attempts128/partials128/pins1024/inflight256/custody256 metadata=16MiB sizes={sizes:?} repetitions={repetitions} warm_probes={warm}; cold=first probe after fresh population; warm=repeated probe same owner; bytes=World charges only");
    for axis in Axis::ALL {
        let mut baseline = None;
        let mut publication_baseline = None;
        for size in sizes {
            let mut cold = vec![];
            let mut warmed = vec![];
            let mut publications = vec![];
            for _ in 0..repetitions {
                let mut population = Population::build(axis, size);
                println!(
                    "profile={name} requested={axis:?}/{size} actual_B_H_U_A_P_O={:?}",
                    population.axes()
                );
                let (elapsed, work) = observe_probe(&population);
                cold.push(elapsed);
                assert_eq!(
                    *baseline.get_or_insert(work),
                    work,
                    "zero structural slope across population"
                );
                for _ in 0..warm {
                    let (elapsed, repeated) = observe_probe(&population);
                    assert_eq!(work, repeated);
                    warmed.push(elapsed);
                }
                let (time, costs) = publication_probe(&mut population);
                publications.push(time);
                assert_eq!(*publication_baseline.get_or_insert(costs), costs);
                population.finish();
            }
            println!("profile={name} axis={axis:?} size={size} structural={baseline:?} slope_per_probe=0");
            println!("profile={name} axis={axis:?} publication_per_attempt={publication_baseline:?} structural_slope=0");
            distribution(
                name,
                &format!("{axis:?}/{size}"),
                "publication after observation probes",
                &mut publications,
            );
            distribution(name, &format!("{axis:?}/{size}"), "cold", &mut cold);
            distribution(name, &format!("{axis:?}/{size}"), "warm", &mut warmed);
        }
    }
    let mut baseline = None;
    for width in [1, 4, 16] {
        let mut times = vec![];
        for _ in 0..repetitions {
            let (time, cost) = writers::run(width);
            times.push(time);
            assert_eq!(*baseline.get_or_insert(cost), cost);
        }
        println!("profile={name} W={width} per_attempt={baseline:?} concurrent_total_signal_contacts={width} concurrent_total_CAS_wins={width} per_attempt_structural_slope=0 total_slope=linear");
        distribution(
            name,
            &format!("W/{width}"),
            "fresh owner concurrent batch",
            &mut times,
        );
    }
}
#[test]
#[ignore = "scheduled Court scale profile"]
fn court_profile() {
    profile("Court", [1, 4, 8], 3, 4);
}
#[test]
#[ignore = "scheduled Standard scale profile"]
fn standard_profile() {
    profile("Standard", [1, 8, 32], 3, 6);
}
#[test]
#[ignore = "scheduled Scale profile and longer model sequence"]
fn scale_profile() {
    profile("Scale", [1, 32, 96], 3, 8);
    super::super::model::driver::run(0x9172cafe, 32);
}
