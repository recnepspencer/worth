use crate::AuthorityMarker;

/// Descriptor binding admitted and the serving incarnation made durable.
#[derive(Debug)]
pub struct HomeAdmitted<Custody: AuthorityMarker> {
    custody: Custody,
}

impl<Custody: AuthorityMarker> HomeAdmitted<Custody> {
    pub fn from_owner(custody: Custody) -> Self {
        Self { custody }
    }

    pub fn consume<Output>(self, readmit: impl FnOnce(Custody) -> Output) -> Output {
        readmit(self.custody)
    }
}
