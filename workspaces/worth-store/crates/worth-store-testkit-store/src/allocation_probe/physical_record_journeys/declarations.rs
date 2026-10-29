pub use std::alloc::System;
pub use std::sync::Arc;
pub use {
    std::sync::atomic::AtomicBool, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
};

pub use {
    stats_alloc::Region, stats_alloc::Stats, stats_alloc::StatsAlloc,
    stats_alloc::INSTRUMENTED_SYSTEM,
};

pub static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

pub fn allocations_during<T>(operation: impl FnOnce() -> T) -> (T, Stats) {
    let region = stats_alloc::Region::new(ALLOCATOR);
    let result = operation();
    (result, region.change())
}

pub fn peak_live_bytes_during<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    let baseline = live_bytes(ALLOCATOR.stats());
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let peak = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(baseline));
    let sampler_running = std::sync::Arc::clone(&running);
    let sampler_peak = std::sync::Arc::clone(&peak);
    let sampler = std::thread::spawn(move || {
        while sampler_running.load(std::sync::atomic::Ordering::Acquire) {
            sampler_peak.fetch_max(
                live_bytes(ALLOCATOR.stats()),
                std::sync::atomic::Ordering::AcqRel,
            );
            std::thread::yield_now();
        }
        sampler_peak.fetch_max(
            live_bytes(ALLOCATOR.stats()),
            std::sync::atomic::Ordering::AcqRel,
        );
    });
    let result = operation();
    peak.fetch_max(
        live_bytes(ALLOCATOR.stats()),
        std::sync::atomic::Ordering::AcqRel,
    );
    running.store(false, std::sync::atomic::Ordering::Release);
    sampler.join().unwrap();
    (
        result,
        peak.load(std::sync::atomic::Ordering::Acquire)
            .saturating_sub(baseline),
    )
}

pub fn live_bytes(stats: Stats) -> usize {
    let net = stats.bytes_allocated as i128 - stats.bytes_deallocated as i128
        + stats.bytes_reallocated as i128;
    net.max(0) as usize
}
