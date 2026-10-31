//! Proportional sensitivity checks for the actual instrumented allocator.
use super::measure;

#[test]
fn requested_object_peak_survives_free_after_high_water() {
    let (_, observed) = measure(|| {
        let first = vec![0_u8; 4_096];
        let second = vec![0_u8; 2_048];
        std::hint::black_box((&first, &second));
        drop(first);
        drop(second);
    });
    assert_eq!(observed.peak_bytes, 6_144);
    assert_eq!(observed.live_at_return_bytes, 0);
}

#[test]
fn realloc_reports_the_instrumented_old_and_new_backing_overlap() {
    let (bytes, observed) = measure(|| {
        let mut bytes = vec![1_u8; 1_024];
        bytes.reserve_exact(2_048);
        std::hint::black_box(bytes)
    });
    assert_eq!(bytes.capacity(), 3_072);
    assert_eq!(observed.peak_bytes, 4_096);
    assert_eq!(observed.live_at_return_bytes, 3_072);
}

#[test]
fn stale_group_free_cannot_erase_new_group_live_objects() {
    let stale = measure(|| vec![0_u8; 8_192]).0;
    let ((first, second), observed) = measure(|| {
        let first = vec![0_u8; 3_072];
        drop(stale);
        let second = vec![0_u8; 2_048];
        std::hint::black_box((first, second))
    });
    assert_eq!((first.len(), second.len()), (3_072, 2_048));
    assert_eq!(observed.peak_bytes, 5_120);
    assert_eq!(observed.live_at_return_bytes, 5_120);
}

#[test]
fn unwind_clears_the_active_group_before_a_fresh_measurement() {
    let failed = std::panic::catch_unwind(|| {
        measure(|| std::panic::resume_unwind(Box::new("expected allocation observer unwind")))
    });
    assert!(failed.is_err());
    let idle = {
        let _session = super::SESSION
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        super::state().is_none()
    };
    assert!(idle, "unwinding releases the original active group");
    let (bytes, observed) = measure(|| vec![0_u8; 512]);
    assert_eq!(bytes.len(), 512);
    assert_eq!(observed.peak_bytes, 512);
    assert_eq!(observed.live_at_return_bytes, 512);
}
