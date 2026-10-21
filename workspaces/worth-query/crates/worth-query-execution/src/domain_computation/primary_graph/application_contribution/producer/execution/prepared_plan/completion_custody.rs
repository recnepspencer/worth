//! Completion closes its invocation before the successor interest is released.
use super::*;
use std::cell::Cell;
use std::rc::Rc;

struct RunningClaim(Rc<Cell<bool>>);
impl Drop for RunningClaim {
    fn drop(&mut self) {
        assert!(
            self.0.replace(false),
            "the running claim closes exactly once"
        );
    }
}
struct SuccessorInterest {
    running: Rc<Cell<bool>>,
    released: Rc<Cell<bool>>,
}
impl Drop for SuccessorInterest {
    fn drop(&mut self) {
        assert!(!self.running.get(), "completion closes before its interest");
        assert!(
            !self.released.replace(true),
            "the interest closes exactly once"
        );
    }
}

#[test]
fn completion_closes_before_successor_custody_on_every_exit() {
    for exit in 0..4 {
        let running = Rc::new(Cell::new(true));
        let released = Rc::new(Cell::new(false));
        let claim = RunningClaim(Rc::clone(&running));
        let interest = SuccessorInterest {
            running: Rc::clone(&running),
            released: Rc::clone(&released),
        };
        let plan = OwnerPlan::new(Vec::new(), move |slots| {
            assert!(slots.is_empty());
            if exit == 3 {
                panic!("completion failure");
            }
            drop(claim);
            Ok::<(), ()>(())
        });
        let plan = plan.complete_with(interest, move |completion| {
            if exit == 1 {
                drop(completion);
                Err(())
            } else {
                let (result, custody) = completion.finish();
                drop(custody);
                result
            }
        });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if exit == 0 {
                drop(plan);
                Ok(())
            } else {
                (plan.complete)(Vec::new())
            }
        }));
        assert_eq!(result.is_err(), exit == 3);
        if let Ok(result) = result {
            assert_eq!(result.is_err(), exit == 1);
        }
        assert!(!running.get());
        assert!(released.get());
    }
}
