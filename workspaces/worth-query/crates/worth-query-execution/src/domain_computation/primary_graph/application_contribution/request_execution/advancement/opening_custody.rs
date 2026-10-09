//! A host call cannot acquire a second budget while its advancement is active.
use super::WorthQueryAdvancementDenial;
use std::{cell::Cell, marker::PhantomData, rc::Rc};
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
/// Thread-bound plumbing for admission, never a way to obtain execution authority.
pub(super) struct OpeningCustody {
    _thread: PhantomData<Rc<()>>,
}
impl OpeningCustody {
    pub(super) fn enter() -> Result<Self, WorthQueryAdvancementDenial> {
        ACTIVE.with(|active| {
            if active.replace(true) {
                let denial = WorthQueryAdvancementDenial::NestedOpening;
                #[cfg(any(test, feature = "test-query-execution-observer"))]
                super::observation::record_result::<()>(&Err(denial));
                Err(denial)
            } else {
                Ok(Self {
                    _thread: PhantomData,
                })
            }
        })
    }
}
impl Drop for OpeningCustody {
    fn drop(&mut self) {
        ACTIVE.with(|active| active.set(false));
    }
}
