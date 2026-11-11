//! Prepared derived projection follows the same original committed owner effect.
use crate::branch::owner_services::SignalDurableBranchHandle;

/// Only the actual original driver invokes this after its committed cutover.
pub trait SignalDurableProjectionPublication: Send {
    fn committed(self: Box<Self>, branch: &SignalDurableBranchHandle);
}
