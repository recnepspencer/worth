//! Re-exports shared checkpoint fact/metadata codecs from `worth-query-execution-lineage`.

pub(in crate::domain_computation::primary_graph) use producer_computation::decode as decode_checkpoint_computation;
pub use worth_query_execution_lineage::facade::primary_graph::application_checkpoint::{
    accepted_output, facts, header, metadata_funding, producer_computation, CheckpointDecodeDenial,
    CheckpointMetadataFunding, MAXIMUM_ENTITY_NAME_BYTES, MAXIMUM_PRODUCER_IDENTITY_BYTES,
    MAXIMUM_ROLE_IDENTITY_BYTES,
};
