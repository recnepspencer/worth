pub use {std::time::Duration, std::time::Instant};

pub use {worth_store::physical_runtime::PhysicalWorkCounterStage};

pub fn await_read_signal_cleanup(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let observation = serving.physical_signal_observation().unwrap();
        if observation.active_locality_count() == 0 && observation.active_in_flight_count() == 0 {
            return;
        }
        if std::time::Instant::now() >= deadline {
            let records = serving.physical_work_observer().causal().records();
            let outcomes = records
                .iter()
                .map(|record| (record.backend_operation(), record.derived_completion()))
                .collect::<Vec<_>>();
            panic!(
                "canonical read work retained Signal state after termination: localities={}, in_flight={}, declared={}, terminal={}, settled={}, outcomes={outcomes:?}",
                observation.active_locality_count(),
                observation.active_in_flight_count(),
                serving
                    .physical_work_counters()
                    .total(worth_store::physical_runtime::PhysicalWorkCounterStage::Declared),
                serving
                    .physical_work_counters()
                    .total(worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal),
                records.len(),
            );
        }
        std::thread::yield_now();
    }
}