use worth_runtime_bridge::facade::AdmittedHistoricalTemporalReplayBasis;

fn fake<T>() -> T {
    panic!("private")
}

fn main() {
    let _ = AdmittedHistoricalTemporalReplayBasis {
        historical_temporal_replay_basis_identity: fake(),
        temporal_admission: fake(),
        historical_truth_basis: fake(),
        retained_previous_values: fake(),
        counters: fake(),
        canonical_basis: fake(),
        digest: fake(),
    };
}
