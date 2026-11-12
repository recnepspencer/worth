//! Exact applied producer inputs, transferred once into live-route retention.
//! The historical envelope keeps no strong pin after that transfer. Its deep
//! clones cannot rebind the packet, and codecs omit it. Neither reissues custody.
use super::{CanonicalCommitEnvelope, CanonicalRequiredOutputOrigin};
use crate::{
    history::retention::PublicationBasisRoute,
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock, Weak},
};

// Erase the physical pin inside canonical live metadata. The concrete reader
// retains a root containing canonical envelopes; the trait closes auto-trait
// recursion at that custody boundary without weakening Send or Sync.
pub(crate) trait AppliedPublicationInputAccess: std::fmt::Debug + Send + Sync {
    fn read(&self) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;
}
impl AppliedPublicationInputAccess
    for RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>
{
    fn read(&self) -> &Self {
        self
    }
}

#[derive(Default)]
pub(crate) struct LivePublicationInputs {
    owner: OnceLock<Weak<CanonicalCommitEnvelope>>,
    inputs: Mutex<
        BTreeMap<
            PublicationBasisRoute,
            (
                Arc<CanonicalRequiredOutputOrigin>,
                Arc<dyn AppliedPublicationInputAccess>,
            ),
        >,
    >,
}
impl Clone for LivePublicationInputs {
    fn clone(&self) -> Self {
        // A deep canonical clone is descriptive, not an issuance of live custody.
        Self::default()
    }
}
impl std::fmt::Debug for LivePublicationInputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LivePublicationInputs")
            .field(
                "routes",
                &self.inputs.lock().unwrap_or_else(|e| e.into_inner()).len(),
            )
            .finish()
    }
}
impl PartialEq for LivePublicationInputs {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for LivePublicationInputs {}
impl LivePublicationInputs {
    pub(crate) fn insert(
        &mut self,
        origin: Arc<CanonicalRequiredOutputOrigin>,
        input: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) {
        self.inputs
            .get_mut()
            .unwrap_or_else(|e| e.into_inner())
            .insert(
                PublicationBasisRoute::of(&origin.meaning),
                (origin, Arc::new(input)),
            );
    }
    pub(super) fn bind(&self, owner: &Weak<CanonicalCommitEnvelope>) {
        self.owner.get_or_init(|| owner.clone());
    }
    pub(crate) fn take_verified(
        envelope: &Arc<CanonicalCommitEnvelope>,
        origin: &Arc<CanonicalRequiredOutputOrigin>,
    ) -> Option<Arc<dyn AppliedPublicationInputAccess>> {
        let live = &envelope.coupled_authored_publication.as_ref()?.live_inputs;
        let owner = live.owner.get()?.upgrade()?;
        if !Arc::ptr_eq(&owner, envelope) {
            return None;
        }
        let (issued, read) = live
            .inputs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&PublicationBasisRoute::of(&origin.meaning))?;
        (Arc::ptr_eq(&issued, origin)
            && read
                .read()
                .candidate_identity()
                .proposal()
                .proposed_version_id()
                == envelope.commit.version_id)
            .then_some(read)
    }
}
