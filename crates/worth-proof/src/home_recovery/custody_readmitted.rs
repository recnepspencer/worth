use crate::AuthorityMarker;

/// The actual owner readmitted every held application obligation before service.
/// The payload remains private to its issuing runtime owner.
#[derive(Debug)]
pub struct CustodyReadmitted<Custody: AuthorityMarker> {
    custody: Custody,
}

impl<Custody: AuthorityMarker> CustodyReadmitted<Custody> {
    pub fn from_owner(custody: Custody) -> Self {
        Self { custody }
    }

    pub fn consume<Output>(self, next_owner: impl FnOnce(Custody) -> Output) -> Output {
        next_owner(self.custody)
    }
}
