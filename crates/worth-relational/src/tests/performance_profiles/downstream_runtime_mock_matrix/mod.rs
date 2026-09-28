use super::*;

mod commit_wave;
mod medium_region_wave;
mod mixed_locality_wave;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_downstream_runtime_mock_matrix() {
    let suite = "downstream_runtime_mock_matrix";

    commit_wave::certify_geometry_commit_downstream_wave(suite);
    medium_region_wave::certify_geometry_commit_downstream_medium_region_wave(suite);
    mixed_locality_wave::certify_geometry_commit_downstream_mixed_locality_wave(suite);
}
