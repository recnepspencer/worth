//! The concrete native directory used by recovery plans and checkpoint readers.
use super::RelationalCheckpointDirectory;
pub(crate) use crate::storage::data::directory_source_edge::NativeDirectoryRetirementHandoff;
pub(crate) type NativeCheckpointDirectoryEdge =
    crate::storage::data::directory_source_edge::DirectorySourceEdge<RelationalCheckpointDirectory>;
