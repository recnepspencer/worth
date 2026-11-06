//! Preserve the known publication subject without changing request-stop semantics.
use super::CodecRetirementClosureStop;
use crate::identity::data::EntityId;

#[derive(Debug)]
pub(crate) struct PublicationHistoryFailure {
    pub(crate) source: Option<EntityId>,
    pub(crate) stop: CodecRetirementClosureStop,
}

impl PublicationHistoryFailure {
    pub(super) fn at_source(source: EntityId, stop: CodecRetirementClosureStop) -> Self {
        Self {
            source: Some(source),
            stop,
        }
    }
}

impl From<CodecRetirementClosureStop> for PublicationHistoryFailure {
    fn from(stop: CodecRetirementClosureStop) -> Self {
        Self { source: None, stop }
    }
}
impl From<&str> for PublicationHistoryFailure {
    fn from(detail: &str) -> Self {
        CodecRetirementClosureStop::from(detail).into()
    }
}
impl From<String> for PublicationHistoryFailure {
    fn from(detail: String) -> Self {
        CodecRetirementClosureStop::from(detail).into()
    }
}
impl From<PublicationHistoryFailure> for CodecRetirementClosureStop {
    fn from(failure: PublicationHistoryFailure) -> Self {
        failure.stop
    }
}
