//! A durable Signal owner retains its actual exclusive runtime admission.
use crate::AuthorityMarker;

/// Pure phase; the owning runtime supplies non-cloneable admission custody.
#[derive(Debug)]
pub struct SignalOwnerReadmitted<Admission: AuthorityMarker> {
    admission: Admission,
}
impl<Admission: AuthorityMarker> SignalOwnerReadmitted<Admission> {
    pub fn from_owner(admission: Admission) -> Self {
        Self { admission }
    }
    pub fn admission(&self) -> &Admission {
        &self.admission
    }
}
