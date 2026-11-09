//! A later request may grow retained backing, but cannot acquire its custody.
use super::admit;
use worth_execution::SerialMemoryBudget;

#[test]
fn a_second_request_grows_the_original_account_and_keeps_it_alive() {
    let original = SerialMemoryBudget::new(Some(1 << 20));
    let second = SerialMemoryBudget::new(Some(1 << 20));
    let reserve = |budget: &SerialMemoryBudget, bytes| {
        budget.reserve(bytes).map(Some).map_err(|denial| {
            crate::execution::denial::lease_denial(worth_execution::LeaseDenial::MemoryExhausted(
                denial,
            ))
        })
    };
    let first = admit(None, 64, &mut |bytes| reserve(&original, bytes))
        .unwrap()
        .unwrap();
    let held = original.charged_memory_bytes();
    assert!(held > 0);
    let grown = admit(Some(&first), 32, &mut |bytes| reserve(&second, bytes))
        .unwrap()
        .unwrap();
    let both = original.charged_memory_bytes();
    assert!(
        both > held,
        "the original account pays for additional retained backing"
    );
    assert_eq!(
        second.charged_memory_bytes(),
        0,
        "the later account must not acquire custody"
    );
    assert_eq!(grown.id(), first.id());
    drop(first);
    assert_eq!(original.charged_memory_bytes(), both);
    drop(grown);
    assert_eq!(original.charged_memory_bytes(), 0);
    assert_eq!(second.charged_memory_bytes(), 0);
}
