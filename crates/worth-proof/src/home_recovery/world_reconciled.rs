use crate::AuthorityMarker;
/// Actual Home and World custody after owner-artifact readmission and reconciliation.
/// Concrete production ports require the Home owner's private payload type.
#[derive(Debug)]
pub struct WorldReconciled<Custody: AuthorityMarker> {
    custody: Custody,
}
impl<Custody: AuthorityMarker> WorldReconciled<Custody> {
    pub fn from_owner(custody: Custody) -> Self {
        Self { custody }
    }
    pub fn consume<Output>(self, next: impl FnOnce(Custody) -> Output) -> Output {
        next(self.custody)
    }
}
