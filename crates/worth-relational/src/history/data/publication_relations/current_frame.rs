//! A grouped retirement is assembled before it can certify a creation frame.
use super::{CanonicalCommitEnvelope, Stop};
use crate::history::data::canonical_commit_envelope::ValidatedOutputOrigins;
use worth_execution::MapKernelContext;

#[derive(Clone, Copy)]
pub(crate) enum CurrentPublicationFrame<'a> {
    Complete(ValidatedOutputOrigins<'a>),
    /// Read-only latest-write corroboration is valid while the group is being
    /// assembled. Any actual birth lookup must still pass the envelope validator.
    Retiring(&'a CanonicalCommitEnvelope),
}

impl<'a> CurrentPublicationFrame<'a> {
    pub(super) fn envelope(self) -> &'a CanonicalCommitEnvelope {
        match self {
            Self::Complete(view) => view.envelope(),
            Self::Retiring(envelope) => envelope,
        }
    }

    pub(super) fn origins(
        self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<ValidatedOutputOrigins<'a>, Stop> {
        match self {
            Self::Complete(view) => Ok(view),
            Self::Retiring(envelope) => envelope.validate_output_origins(context),
        }
    }
}
