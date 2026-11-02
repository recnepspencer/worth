use crate::AuthorityMarker;

/// Original-source initialization permission, before any reconstructed receiver
/// can exist. The concrete Native payload binds its actual runtime and source.
#[derive(Debug)]
pub struct FreshNativeInitialization<Custody: AuthorityMarker> {
    custody: Custody,
}
impl<Custody: AuthorityMarker> FreshNativeInitialization<Custody> {
    pub fn from_owner(custody: Custody) -> Self {
        Self { custody }
    }
    pub fn consume<Output>(self, next: impl FnOnce(Custody) -> Output) -> Output {
        next(self.custody)
    }
}
