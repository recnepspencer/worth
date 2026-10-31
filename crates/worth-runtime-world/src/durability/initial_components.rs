//! Transport each owner's mandatory initial-component recipe without copying its schema.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedWorldInitialComponents {
    pub(crate) relational: worth_relational::facade::branch::RecordedRelationalComponentReference,
    pub(crate) signal: worth_signal::facade::branch::RecordedSignalInitialEmptyRoot,
    pub(crate) bridge: worth_runtime_bridge::facade::RecordedRuntimeWorldBaseline,
}
