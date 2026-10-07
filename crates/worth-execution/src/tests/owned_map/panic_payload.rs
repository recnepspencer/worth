use super::{
    authority, map, request, ChargedBytes, MapKernelFailure, MapOutcome, MapStop, TEST_LOCK,
};
use std::{
    panic::{catch_unwind, panic_any, AssertUnwindSafe},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

struct Payload(Arc<AtomicUsize>);
impl Drop for Payload {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
        panic_any(SecondPayload);
    }
}
struct SecondPayload;
impl Drop for SecondPayload {
    fn drop(&mut self) {
        panic!("second caught payload must not be dropped");
    }
}
struct Input(Arc<AtomicUsize>, Arc<AtomicUsize>);
impl ChargedBytes for Input {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
        panic_any(Payload(Arc::clone(&self.1)));
    }
}

#[test]
fn owned_destructor_contains_panicking_payload_on_every_cleanup_path() {
    let _lock = TEST_LOCK.lock().unwrap();
    for count in [1, 2] {
        for path in 0..3 {
            let source = crate::CancellationSource::new();
            let mut req = request(1, if path == 2 { 1 } else { 2_000 }, 100);
            req.cancellation = source.token();
            if path == 1 {
                source.cancel();
            }
            let lease = authority().request_lease(req).unwrap();
            let drops = Arc::new(AtomicUsize::new(0));
            let payloads = Arc::new(AtomicUsize::new(0));
            let result =
                catch_unwind(AssertUnwindSafe(|| {
                    map((0..count).map(|_| Input(Arc::clone(&drops), Arc::clone(&payloads))))
                        .run_owned(Some(&lease), |input, _| {
                            drop(input);
                            Ok::<_, MapKernelFailure<()>>(())
                        })
                }));
            let result = result.unwrap_or_else(|payload| {
                std::mem::forget(payload);
                panic!("caught payload escaped cleanup path {path}");
            });
            assert!(matches!(
                result,
                MapOutcome::Stopped {
                    reason: MapStop::Failure {
                        cause: MapKernelFailure::Panic,
                        ..
                    },
                    ..
                }
            ));
            assert_eq!(drops.load(Ordering::SeqCst), count);
            assert_eq!(payloads.load(Ordering::SeqCst), count);
            assert!(lease
                .reserve_memory(lease.policy().budget().charged_memory_bytes())
                .is_ok());
        }
    }
}

#[test]
fn both_kernel_modes_contain_panicking_payload() {
    let _lock = TEST_LOCK.lock().unwrap();
    for owned in [false, true] {
        let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
        let payloads = Arc::new(AtomicUsize::new(0));
        let result = catch_unwind(AssertUnwindSafe(|| {
            if owned {
                map([0_u64]).run_owned(Some(&lease), |_, _| {
                    panic_any(Payload(Arc::clone(&payloads)));
                    #[allow(unreachable_code)]
                    Ok::<_, MapKernelFailure<()>>(())
                })
            } else {
                map([0_u64]).run(Some(&lease), |_, _| {
                    panic_any(Payload(Arc::clone(&payloads)));
                    #[allow(unreachable_code)]
                    Ok::<_, MapKernelFailure<()>>(())
                })
            }
        }));
        let result = result.unwrap_or_else(|payload| {
            std::mem::forget(payload);
            panic!("caught kernel payload escaped: owned={owned}");
        });
        assert!(matches!(
            result,
            MapOutcome::Stopped {
                reason: MapStop::Failure {
                    cause: MapKernelFailure::Panic,
                    ..
                },
                ..
            }
        ));
        assert_eq!(payloads.load(Ordering::SeqCst), 1);
    }
}
