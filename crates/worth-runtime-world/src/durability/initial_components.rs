//! Each owner supplies its own original branch recipe; Signal remains opaque bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecordedWorldInitialComponents {
    pub(crate) relational: worth_relational::facade::branch::RecordedRelationalComponentReference,
    pub(crate) signal: super::artifact::WorldSignalEvidence,
    pub(crate) bridge: worth_runtime_bridge::facade::RecordedRuntimeWorldBaseline,
}
