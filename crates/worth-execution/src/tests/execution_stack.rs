//! Public physical-entry callback semantics, with no fabricated platform authority.
use crate::run_with_execution_stack;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn execution_stack_preserves_non_send_borrows_results_and_the_caller_thread() {
    let value = Rc::new(Cell::new(0));
    let caller = std::thread::current().id();
    let result: Result<Rc<Cell<i32>>, std::io::ErrorKind> = run_with_execution_stack(|| {
        assert_eq!(std::thread::current().id(), caller);
        value.set(7);
        Err(std::io::ErrorKind::Interrupted)
    });
    assert_eq!(result.unwrap_err(), std::io::ErrorKind::Interrupted);
    assert_eq!(value.get(), 7);
    let mut values = [1, 2];
    let borrowed = run_with_execution_stack(|| &mut values[..]);
    borrowed[0] = 3;
    assert_eq!(values, [3, 2]);
    let owned = run_with_execution_stack(|| Rc::clone(&value));
    assert!(Rc::ptr_eq(&owned, &value));
}

struct DropObservation(Rc<Cell<usize>>);
impl Drop for DropObservation {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn execution_stack_unwind_drops_the_original_callback_value_once() {
    let drops = Rc::new(Cell::new(0));
    let observed = DropObservation(Rc::clone(&drops));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_with_execution_stack::<()>(move || {
            let _observed = observed;
            panic!("original callback unwind");
        });
    }));
    assert!(result.is_err());
    assert_eq!(drops.get(), 1);
}
