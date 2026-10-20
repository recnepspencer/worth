use super::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

struct Watched {
    bytes: Vec<u8>,
    ledger: crate::SerialMemoryBudget,
    observed: Arc<AtomicU64>,
}
impl ChargedBytes for Watched {
    fn additional_charged_bytes(&self) -> u64 {
        self.bytes.additional_charged_bytes()
    }
}
impl Drop for Watched {
    fn drop(&mut self) {
        self.observed
            .store(self.ledger.charged_memory_bytes(), Ordering::SeqCst);
    }
}

#[test]
fn refused_opaque_payload_is_disposed_before_its_original_scope_credit() {
    let serial =
        SerialRequest::from_policy(&policy(Some(128 * 1024)), CancellationToken::new(), None);
    let other_memory = serial.memory().reserve(96 * 1024).unwrap();
    let other_value = vec![41_u8; 96 * 1024];
    let mut memory = serial.memory().reserve(0).unwrap();
    let before = AtomicU64::new(0);
    let observed = Arc::new(AtomicU64::new(0));
    let stopped = ExecutionRequest::serial(&serial).run_measured_result(
        ExecutionWorkCeiling::new(None),
        0,
        &mut memory,
        |_, _| {
            before.store(serial.memory().charged_memory_bytes(), Ordering::SeqCst);
            Ok::<_, MapKernelFailure<()>>(Watched {
                bytes: vec![7_u8; 96 * 1024],
                ledger: serial.memory().clone(),
                observed: Arc::clone(&observed),
            })
        },
    );
    assert!(matches!(
        stopped.result,
        Err(ScopeStop::Admission(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial::Capacity { .. }
        )))
    ));
    assert_eq!(
        observed.load(Ordering::SeqCst),
        before.load(Ordering::SeqCst)
    );
    assert!(
        observed.load(Ordering::SeqCst) > other_memory.bytes(),
        "the original scope charge was live at Drop"
    );
    assert_eq!(memory.bytes(), 0);
    assert_eq!(serial.memory().charged_memory_bytes(), other_memory.bytes());
    assert!(other_value.iter().all(|byte| *byte == 41));
    drop(other_value);
    drop(other_memory);
    assert_eq!(serial.memory().charged_memory_bytes(), 0);
}

#[test]
fn cancelled_callback_payload_drops_while_its_before_growth_charge_is_held() {
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_policy(&policy(None), cancellation.token(), None);
    let mut memory = serial.memory().reserve(0).unwrap();
    let before = AtomicU64::new(0);
    let observed = Arc::new(AtomicU64::new(0));
    let stopped = ExecutionRequest::serial(&serial).run_measured_result(
        ExecutionWorkCeiling::new(None),
        0,
        &mut memory,
        |context, result_memory| {
            result_memory.resize(1_024).unwrap();
            let value = Watched {
                bytes: vec![11_u8; 1_024],
                ledger: serial.memory().clone(),
                observed: Arc::clone(&observed),
            };
            before.store(serial.memory().charged_memory_bytes(), Ordering::SeqCst);
            cancellation.cancel();
            assert_eq!(context.checkpoint(0), Err(MapKernelStop::Cancelled));
            Ok::<_, MapKernelFailure<()>>(value)
        },
    );
    assert!(matches!(
        stopped.result,
        Err(ScopeStop::Failure(MapKernelFailure::Stop(
            MapKernelStop::Cancelled
        )))
    ));
    assert_eq!(
        observed.load(Ordering::SeqCst),
        before.load(Ordering::SeqCst)
    );
    assert!(observed.load(Ordering::SeqCst) >= 1_024);
    assert_eq!(memory.bytes(), 0);
    assert_eq!(serial.memory().charged_memory_bytes(), 0);
}
