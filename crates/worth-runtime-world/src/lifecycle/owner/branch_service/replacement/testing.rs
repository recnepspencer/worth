//! Receiver-scoped pause after the actual effect, before original handoff.
use crate::identity::RuntimeWorldOwnerIdentity;
use std::cell::RefCell;

type Hook = (RuntimeWorldOwnerIdentity, Box<dyn FnOnce()>);
thread_local! { static AFTER: RefCell<Option<Hook>> = const { RefCell::new(None) }; }

pub(in crate::lifecycle::owner::branch_service) struct Rehearsal(Option<Hook>);

pub(in crate::lifecycle::owner::branch_service) fn rehearse(
    owner: RuntimeWorldOwnerIdentity,
    hook: impl FnOnce() + 'static,
) -> Rehearsal {
    Rehearsal(AFTER.with(|slot| slot.replace(Some((owner, Box::new(hook))))))
}

pub(super) fn after_retirement(owner: RuntimeWorldOwnerIdentity) {
    let hook = AFTER.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|(receiver, _)| *receiver == owner)
        {
            slot.take()
        } else {
            None
        }
    });
    if let Some((_, hook)) = hook {
        hook();
    }
}

impl Drop for Rehearsal {
    fn drop(&mut self) {
        AFTER.with(|slot| {
            slot.replace(self.0.take());
        });
    }
}
