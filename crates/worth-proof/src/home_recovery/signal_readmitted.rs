//! Signal authored admission precedes World reconciliation on the same Home.
use crate::AuthorityMarker;

/// The actual Signal durable owner and its fresh Bridge installation have
/// readmitted every authored branch. The concrete Home payload retains them;
/// neither arbitrary callback success nor a descriptive image grants this phase.
///
/// ```compile_fail,E0308
/// use worth_proof::{AuthorityMarker, RelationalReadmitted, SignalReadmitted};
/// struct Custody;
/// impl AuthorityMarker for Custody {}
/// fn reconcile(_: SignalReadmitted<Custody>) {}
/// reconcile(RelationalReadmitted::from_owner(Custody));
/// ```
#[derive(Debug)]
pub struct SignalReadmitted<Custody: AuthorityMarker> {
    custody: Custody,
}
impl<Custody: AuthorityMarker> SignalReadmitted<Custody> {
    pub fn from_owner(custody: Custody) -> Self {
        Self { custody }
    }
    pub fn consume<Output>(self, next: impl FnOnce(Custody) -> Output) -> Output {
        next(self.custody)
    }
}
