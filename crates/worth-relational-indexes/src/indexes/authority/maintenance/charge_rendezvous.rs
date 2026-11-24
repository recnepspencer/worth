//! A synchronous observation scoped to one owned maintenance pass.

#[derive(Clone, Copy)]
pub struct ChargeRendezvous<'scope>(&'scope dyn Fn());

impl<'scope> ChargeRendezvous<'scope> {
    #[doc(hidden)]
    pub fn observe_charges_for_test(observe: &'scope dyn Fn()) -> Self {
        Self(observe)
    }

    pub(super) fn meet(self) {
        (self.0)();
    }
}
