use worth_signal::facade::core::AsyncNodeMilestoneDPerformanceCloseout;

fn fake<T>() -> T {
    panic!("type-check only")
}

fn main() {
    let closeout: AsyncNodeMilestoneDPerformanceCloseout = fake();
    let _ = closeout.rows;
}
