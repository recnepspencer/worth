use super::*;

#[test]
fn owner_context_restores_after_nested_work() {
    assert!(!ExecutionOwnerContext::is_active());
    let owner = ExecutionOwnerContext::enter();
    let inherited = CapturedOwnerContext::capture();
    {
        let _nested = inherited.enter();
        assert!(ExecutionOwnerContext::is_active());
    }
    assert!(
        ExecutionOwnerContext::is_active(),
        "nested work restores the owning frame"
    );
    drop(owner);
    assert!(
        !ExecutionOwnerContext::is_active(),
        "owner unwind cannot mark later work"
    );
}

crate::needs_os_threads! {
#[test]
fn descendant_unwind_releases_custody_before_later_work() {
    let absent = CapturedOwnerContext::capture();
    let owner = ExecutionOwnerContext::enter();
    let inherited = CapturedOwnerContext::capture();
    drop(owner);
    std::thread::scope(|scope| {
        scope.spawn(move || {
            assert!(!ExecutionOwnerContext::is_active());
            assert!(std::panic::catch_unwind(|| {
                let _work = inherited.enter();
                assert!(ExecutionOwnerContext::is_active(), "a descendant inherits its owner");
                panic!("descendant work unwinds");
            }).is_err());
            assert!(!ExecutionOwnerContext::is_active(), "unwind releases descendant custody");
            let _unowned = absent.enter();
            assert!(!ExecutionOwnerContext::is_active(), "later work has only its own context");
        });
    });
}
}
