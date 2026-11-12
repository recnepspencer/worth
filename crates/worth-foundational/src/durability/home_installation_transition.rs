//! Installation meaning from a completed Home transition, without live authority.
use super::{ApplicationHomeBinding, HomeProgramTransitionTargetMeaning};
use crate::CanonicalDigestId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeProgramTransitionInstallationBinding {
    origin: [CanonicalDigestId; 2],
    successor: [CanonicalDigestId; 3],
    target: HomeProgramTransitionTargetMeaning,
}
impl HomeProgramTransitionInstallationBinding {
    /// Descriptive DATA. Only the attached Home's concrete proof can admit it.
    pub fn from_recorded(
        origin: &ApplicationHomeBinding,
        successor: &ApplicationHomeBinding,
        target: HomeProgramTransitionTargetMeaning,
    ) -> Self {
        Self {
            origin: [origin.package(), origin.schema()],
            successor: [
                successor.package(),
                successor.schema(),
                successor.selected().revision(),
            ],
            target,
        }
    }
    pub const fn origin(&self) -> [CanonicalDigestId; 2] {
        self.origin
    }
    pub const fn successor(&self) -> [CanonicalDigestId; 3] {
        self.successor
    }
    pub const fn target(&self) -> HomeProgramTransitionTargetMeaning {
        self.target
    }
}
