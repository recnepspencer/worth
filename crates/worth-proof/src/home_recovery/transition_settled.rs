use crate::AuthorityMarker;

/// The actual opening transition reached its durable settled disposition before service.
/// The payload remains private to its issuing runtime owner.
#[derive(Debug)]
pub struct TransitionSettled<Custody: AuthorityMarker> {
    custody: Custody,
}

impl<Custody: AuthorityMarker> TransitionSettled<Custody> {
    pub fn from_owner(custody: Custody) -> Self {
        Self { custody }
    }

    pub fn consume<Output>(self, next_owner: impl FnOnce(Custody) -> Output) -> Output {
        next_owner(self.custody)
    }
}
