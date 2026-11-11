use crate::AuthorityMarker;

/// Query readmitted its installed meaning, actual source, publication and World selection.
/// The payload remains private to its issuing runtime owner.
#[derive(Debug)]
pub struct QueryReadmitted<Custody: AuthorityMarker> {
    custody: Custody,
}

impl<Custody: AuthorityMarker> QueryReadmitted<Custody> {
    pub fn from_owner(custody: Custody) -> Self {
        Self { custody }
    }

    pub fn consume<Output>(self, next_owner: impl FnOnce(Custody) -> Output) -> Output {
        next_owner(self.custody)
    }
}
