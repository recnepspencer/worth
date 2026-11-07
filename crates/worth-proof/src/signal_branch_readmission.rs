//! A descriptor description cannot replace the receiving Signal owner's admission.
use crate::AuthorityMarker;

/// Pure readmission phase. The runtime payload has a sealed constructor and
/// owns the actual branch lease and current incarnation fence.
///
/// ```compile_fail,E0308
/// use worth_proof::{AuthorityMarker, SignalBranchReadmitted};
/// struct ActualOwnerHandle;
/// impl AuthorityMarker for ActualOwnerHandle {}
/// struct Description;
/// impl AuthorityMarker for Description {}
/// fn read(_: &SignalBranchReadmitted<ActualOwnerHandle>) {}
/// read(&SignalBranchReadmitted::from_owner(Description));
/// ```
#[derive(Debug)]
pub struct SignalBranchReadmitted<Admission: AuthorityMarker> {
    admission: Admission,
}
impl<Admission: AuthorityMarker> SignalBranchReadmitted<Admission> {
    pub fn from_owner(admission: Admission) -> Self {
        Self { admission }
    }
    pub fn admission(&self) -> &Admission {
        &self.admission
    }
    pub fn consume<Output>(self, use_admission: impl FnOnce(Admission) -> Output) -> Output {
        use_admission(self.admission)
    }
}
