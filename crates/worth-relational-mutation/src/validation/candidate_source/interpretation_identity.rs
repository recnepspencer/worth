use super::NativeManagedSourceInterpretation;
use std::any::Any;
use std::sync::{Arc, Weak};

trait NativeSourceInterpretationBacking: std::fmt::Debug + Send + Sync {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl NativeSourceInterpretationBacking for NativeManagedSourceInterpretation {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

/// Receipts compare the actual owner identity without keeping its roots alive.
/// Upgrading this weak link is not Source admission; installed audience and the
/// exact actual observation still have to be checked by the receiving owner.
#[derive(Clone, Debug)]
pub struct NativeSourceInterpretationIdentity(Weak<dyn NativeSourceInterpretationBacking>);

impl NativeSourceInterpretationIdentity {
    pub(super) fn from_owner(owner: &Arc<NativeManagedSourceInterpretation>) -> Self {
        let erased: Arc<dyn NativeSourceInterpretationBacking> =
            Arc::<NativeManagedSourceInterpretation>::clone(owner);
        Self(Arc::downgrade(&erased))
    }

    pub(crate) fn owner(&self) -> Option<Arc<NativeManagedSourceInterpretation>> {
        self.0.upgrade().map(|backing| {
            backing
                .into_any()
                .downcast::<NativeManagedSourceInterpretation>()
                .expect(
                    "NativeSourceInterpretationIdentity always wraps NativeManagedSourceInterpretation",
                )
        })
    }
}

impl PartialEq for NativeSourceInterpretationIdentity {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(self.0.as_ptr(), other.0.as_ptr())
    }
}
impl Eq for NativeSourceInterpretationIdentity {}
