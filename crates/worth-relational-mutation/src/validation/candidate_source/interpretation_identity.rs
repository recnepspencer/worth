use super::NativeManagedSourceInterpretation;
use std::sync::{Arc, Weak};

/// Receipts compare the actual owner identity without keeping its roots alive.
/// Upgrading this weak link is not Source admission; installed audience and the
/// exact actual observation still have to be checked by the receiving owner.
#[derive(Clone, Debug)]
pub struct NativeSourceInterpretationIdentity(Weak<NativeManagedSourceInterpretation>);

impl NativeSourceInterpretationIdentity {
    pub(super) fn from_owner(owner: &Arc<NativeManagedSourceInterpretation>) -> Self {
        Self(Arc::downgrade(owner))
    }

    pub(crate) fn owner(&self) -> Option<Arc<NativeManagedSourceInterpretation>> {
        self.0.upgrade()
    }
}

impl PartialEq for NativeSourceInterpretationIdentity {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for NativeSourceInterpretationIdentity {}
