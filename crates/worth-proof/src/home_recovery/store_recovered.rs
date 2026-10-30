use crate::AuthorityMarker;

/// Physical initialization/recovery completed under the continuously held lease.
#[derive(Debug)]
pub struct StoreRecovered<Custody: AuthorityMarker> {
    custody: Custody,
}

impl<Custody: AuthorityMarker> StoreRecovered<Custody> {
    /// Only the physical owner can construct the concrete custody payload.
    pub fn from_owner(custody: Custody) -> Self {
        Self { custody }
    }

    pub fn consume<Output>(self, admit: impl FnOnce(Custody) -> Output) -> Output {
        admit(self.custody)
    }
}
