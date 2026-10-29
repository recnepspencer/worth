#[test]
#[ignore]
fn elapsed_work_has_a_cost_category() {
    let started = std::time::Instant::now();
    assert!(started.elapsed() < std::time::Duration::from_secs(1));
}
