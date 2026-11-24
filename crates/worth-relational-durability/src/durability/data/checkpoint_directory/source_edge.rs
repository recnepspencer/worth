//! The concrete native directory used by recovery plans and checkpoint readers.
use super::RelationalCheckpointDirectory;
pub type NativeCheckpointDirectoryEdge =
    worth_relational_storage::internal_port::storage::data::directory_source_edge::DirectorySourceEdge<RelationalCheckpointDirectory>;
