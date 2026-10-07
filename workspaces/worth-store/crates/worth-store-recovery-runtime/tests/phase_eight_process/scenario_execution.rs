/// Each scenario owns its mutable root and process lifecycle. Bound the number
/// of worlds in flight without imposing an order on independent crash seams.
pub(crate) fn run_independent_scenarios<T: Sync>(scenarios: &[T], run: impl Fn(usize, &T) + Sync) {
    const IN_FLIGHT: usize = 4;
    for (batch, scenarios) in scenarios.chunks(IN_FLIGHT).enumerate() {
        std::thread::scope(|scope| {
            for (offset, scenario) in scenarios.iter().enumerate() {
                let run = &run;
                scope.spawn(move || run(batch * IN_FLIGHT + offset, scenario));
            }
        });
    }
}
