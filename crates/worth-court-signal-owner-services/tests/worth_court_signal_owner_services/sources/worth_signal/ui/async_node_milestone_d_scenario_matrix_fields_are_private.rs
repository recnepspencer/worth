use worth_signal::facade::core::AsyncNodeMilestoneDScenarioMatrix;

fn fake<T>() -> T {
    panic!("type-check only")
}

fn main() {
    let matrix: AsyncNodeMilestoneDScenarioMatrix = fake();
    let _ = matrix.rows;
}
