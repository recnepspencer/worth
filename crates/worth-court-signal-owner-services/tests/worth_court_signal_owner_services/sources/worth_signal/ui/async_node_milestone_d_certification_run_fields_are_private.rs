use worth_signal::facade::core::AsyncNodeMilestoneDCertificationRun;

fn fake<T>() -> T {
    panic!("type-check only")
}

fn main() {
    let run: AsyncNodeMilestoneDCertificationRun = fake();
    let _ = run.scenario_matrix;
}
