//! Carrier allocation lifetime/refusal evidence, not Source or World authority.
use super::*;
#[derive(Debug)]
struct Opaque {
    marker: [u64; 8],
    drops: Arc<AtomicUsize>,
}
impl Drop for Opaque {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
fn request(bytes: u64) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(bytes)),
        CancellationSource::new().token(),
        None,
    )
}

#[test]
fn context_weak_receipts_keep_original_inline_charge_after_last_strong_payload_closes() {
    let host = request(65_536);
    let execution = ExecutionRequest::serial(&host);
    let memory = host.memory().unwrap();
    let baseline = memory.charged_memory_bytes();
    let drops = Arc::new(AtomicUsize::new(0));
    let strong = ManagedIndexSourceContextShare::new(
        &execution,
        Opaque {
            marker: [17; 8],
            drops: drops.clone(),
        },
    )
    .unwrap();
    assert_eq!(strong.value().marker, [17; 8]);
    let charged = memory.charged_memory_bytes();
    assert!(charged > baseline);
    let second = strong.clone();
    let receipt = strong.receipt();
    let cloned = receipt.clone();
    let loan = receipt.upgrade().unwrap();
    assert_eq!(loan.downcast_ref::<Opaque>().unwrap().marker, [17; 8]);
    assert_eq!(
        memory.charged_memory_bytes(),
        charged,
        "sharing has no new backing allocation"
    );
    drop(loan);
    drop(strong);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(second);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(receipt.upgrade().is_none());
    assert_eq!(
        memory.charged_memory_bytes(),
        charged,
        "weak custody still retains the entire inline allocation"
    );
    drop(receipt);
    assert_eq!(memory.charged_memory_bytes(), charged);
    drop(cloned);
    assert_eq!(memory.charged_memory_bytes(), baseline);
}

#[test]
fn context_allocation_refusal_returns_the_opaque_input_without_running_its_drop() {
    let host = request(0);
    let execution = ExecutionRequest::serial(&host);
    let drops = Arc::new(AtomicUsize::new(0));
    let result = ManagedIndexSourceContextShare::new(
        &execution,
        Opaque {
            marker: [23; 8],
            drops: drops.clone(),
        },
    );
    let (_, returned) = match result {
        Err(rejected) => rejected,
        Ok(_) => panic!("zero original capacity must refuse"),
    };
    assert_eq!(returned.marker, [23; 8]);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    drop(returned); // The caller chooses the guard-free retirement boundary.
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
