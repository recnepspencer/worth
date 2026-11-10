use super::*;

#[test]
fn live_slots_are_ordered_and_terminal_slots_are_removed() {
    let registry = WorthQueryRecoveryHandleRegistry::new();
    let first = registry.register_axis_probe();
    let middle = registry.register_axis_probe();
    let last = registry.register_axis_probe();
    assert!(registry.mark_terminal(middle, WorthQueryRecoveryResourceTerminal::Consumed));
    assert_eq!(registry.enumerate_live(), vec![first, last]);
    assert!(registry.is_live(first));
    assert!(!registry.is_live(middle));
    let entrant = registry.register_axis_probe();
    assert_eq!(registry.enumerate_live(), vec![first, last, entrant]);
    for slot in [last, first, entrant] {
        assert!(registry.mark_terminal(slot, WorthQueryRecoveryResourceTerminal::Disposed));
    }
    registry.assert_no_live_handles();
}
