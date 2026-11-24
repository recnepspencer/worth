//! Exact applied producer inputs, transferred once into live-route retention.
//! The historical envelope keeps no strong pin after that transfer. Its deep
//! clones cannot rebind the packet, and codecs omit it. Neither reissues custody.
use branch::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_branch::internal_port as branch;
pub mod route_image;
use crate::visibility::materialization::read_records::reader::pinned_read::RelationalAppliedCandidateReadIdentity;
use crate::visibility::materialization::read_records::reader::pinned_read::RelationalPinnedRead;
use branch::history::retention::publication_basis::PublicationBasisRoute;
use std::{
    collections::BTreeMap,
    sync::{Arc, Weak},
};

use branch::history::retention::publication_basis::RetainedAppliedPublicationPin;

type AppliedPublicationRead = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;

// The trait object breaks auto-trait recursion through weak root route images.
impl RetainedAppliedPublicationPin for AppliedPublicationRead {}

pub(crate) fn retained_applied_read(
    pin: &dyn RetainedAppliedPublicationPin,
) -> &AppliedPublicationRead {
    let original: &dyn std::any::Any = pin;
    original
        .downcast_ref::<AppliedPublicationRead>()
        .expect("the publication owner retained its original applied read")
}

/// Applied inputs collected by the original output installation.
#[derive(Debug, Default)]
pub(crate) struct LivePublicationInputs {
    routes: BTreeMap<
        PublicationBasisRoute,
        (
            Arc<CanonicalRequiredOutputOrigin>,
            Arc<dyn RetainedAppliedPublicationPin>,
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
    ) -> Option<Arc<dyn RetainedAppliedPublicationPin>> {
        if !Weak::ptr_eq(&self.owner, &Arc::downgrade(envelope)) {
            return None;
        }
        let (issued, read) = self
            .inputs
            .routes
            .remove(&PublicationBasisRoute::of(&origin.meaning))?;
        (Arc::ptr_eq(&issued, origin)
            && retained_applied_read(read.as_ref())
                .candidate_identity()
                .proposal()
                .proposed_version_id()
                == envelope.commit.version_id)
            .then_some(read)
    }
}
