//! Exact applied producer inputs, transferred once into live-route retention.
//! The historical envelope keeps no strong pin after that transfer. Its deep
//! clones cannot rebind the packet, and codecs omit it. Neither reissues custody.
use crate::history::data::{CanonicalCommitEnvelope, CanonicalRequiredOutputOrigin};
mod route_image;
use crate::{
    history::retention::PublicationBasisRoute,
    visibility::materialization::read_records::reader::pinned_read::{
        RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
    },
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Weak},
};

// The concrete reader retains a root with weak route images. This boundary
// closes auto-trait recursion while retaining typed access to the original pin.
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

/// Applied inputs collected by the original output installation.
#[derive(Debug, Default)]
pub(crate) struct LivePublicationInputs {
    routes: BTreeMap<
        PublicationBasisRoute,
        (
            Arc<CanonicalRequiredOutputOrigin>,
            Arc<dyn AppliedPublicationInputAccess>,
        ),
    >,
}

impl LivePublicationInputs {
    pub(crate) fn insert(
        &mut self,
        origin: Arc<CanonicalRequiredOutputOrigin>,
        input: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) {
        self.routes.insert(
            PublicationBasisRoute::of(&origin.meaning),
            (origin, Arc::new(input)),
        );
    }

    pub(crate) fn bind(
        self,
        envelope: &Arc<CanonicalCommitEnvelope>,
    ) -> BoundLivePublicationInputs {
        BoundLivePublicationInputs {
            owner: Arc::downgrade(envelope),
            inputs: self,
        }
    }
}

/// The route inputs of this exact frozen envelope. Registration consumes this
/// phase and transfers its original pins into the retained route image.
pub(crate) struct BoundLivePublicationInputs {
    owner: Weak<CanonicalCommitEnvelope>,
    inputs: LivePublicationInputs,
}

impl BoundLivePublicationInputs {
    fn take_verified(
        &mut self,
        envelope: &Arc<CanonicalCommitEnvelope>,
        origin: &Arc<CanonicalRequiredOutputOrigin>,
    ) -> Option<Arc<dyn AppliedPublicationInputAccess>> {
        if !Weak::ptr_eq(&self.owner, &Arc::downgrade(envelope)) {
            return None;
        }
        let (issued, read) = self
            .inputs
            .routes
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
