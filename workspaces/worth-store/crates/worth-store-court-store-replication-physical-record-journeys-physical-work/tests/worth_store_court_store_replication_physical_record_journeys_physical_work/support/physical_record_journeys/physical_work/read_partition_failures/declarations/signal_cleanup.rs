use super::*;

pub(crate) fn signal(
    serving: &ServingPhysicalRuntime,
) -> worth_store::physical_runtime::PhysicalSignalObservation {
    serving
        .physical_signal_observation()
        .expect("serving runtime owns Signal")
}

pub(crate) fn await_signal_cleanup(serving: &ServingPhysicalRuntime) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let observation = signal(serving);
        if observation.active_locality_count() == 0 && observation.active_in_flight_count() == 0 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "read route retained Signal state: locality={}, in_flight={}",
            observation.active_locality_count(),
            observation.active_in_flight_count(),
        );
        std::thread::yield_now();
    }
}
