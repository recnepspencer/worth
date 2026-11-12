//! Named verification phase for one owner's original checkpoint inputs.
use crate::AuthorityMarker;

/// Verified original inputs are distinct from a decoded checkpoint description.
/// The concrete runtime payload has private owner-only construction.
///
/// A foreign payload cannot replace that concrete specialization:
/// ```compile_fail,E0308
/// use worth_proof::{AuthorityMarker, NativeCheckpointOriginalsVerified};
/// struct NativeOriginals;
/// impl AuthorityMarker for NativeOriginals {}
/// struct Description;
/// impl AuthorityMarker for Description {}
/// fn capture(_: NativeCheckpointOriginalsVerified<NativeOriginals>) {}
/// capture(NativeCheckpointOriginalsVerified::from_owner(Description));
/// ```
#[derive(Debug)]
pub struct NativeCheckpointOriginalsVerified<Originals: AuthorityMarker> {
    originals: Originals,
}

impl<Originals: AuthorityMarker> NativeCheckpointOriginalsVerified<Originals> {
    pub fn from_owner(originals: Originals) -> Self {
        Self { originals }
    }

    pub fn value(&self) -> &Originals {
        &self.originals
    }

    pub fn consume<Output>(self, next_owner: impl FnOnce(Originals) -> Output) -> Output {
        next_owner(self.originals)
    }
}
