use crate::AuthorityMarker;

/// The one native runtime has readmitted its section and durable tails.
///
/// This is a partial opening stage; it does not grant application service.
#[derive(Debug)]
pub struct RelationalReadmitted<Custody: AuthorityMarker> {
    custody: Custody,
}

impl<Custody: AuthorityMarker> RelationalReadmitted<Custody> {
    pub fn from_owner(custody: Custody) -> Self {
        Self { custody }
    }

    pub fn consume<Output>(self, next_owner: impl FnOnce(Custody) -> Output) -> Output {
        next_owner(self.custody)
    }
}
