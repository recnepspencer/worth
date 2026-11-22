//! Re-exports checkpoint core from `worth-query-execution-lineage` and retains
//! runtime capture/inspection modules in `worth-query-execution`.

pub(crate) mod capture;
mod inspection;

pub use inspection::{
    CheckpointAllocatorInspection, CheckpointRecordFrontier, CheckpointRecordIdentity,
    CheckpointReservationAdvanceDenial, WorthQueryApplicationCheckpointInspection,
    WorthQueryCheckpointInspectionDenial, WorthQueryCheckpointModelBytes,
    WorthQueryCheckpointRecordClass,
};
pub(in crate::domain_computation::primary_graph) use producer_computation::decode as decode_checkpoint_computation;
pub use worth_query_execution_lineage::facade::primary_graph::application_checkpoint::{
    accepted_output, encode, facts, header, interim_transport, metadata_funding,
    producer_computation, CheckpointDecodeDenial, CheckpointMetadataFunding,
    DecodedApplicationCheckpoint, WorthQueryApplicationCheckpoint,
    WorthQueryApplicationCheckpointSectionBytes, WorthQueryCheckpointCaptureDenial,
    WorthQueryCheckpointCapturePolicy, WorthQueryInterimCheckpointOrigin,
    WorthQueryNativeCheckpointSectionBytes, BODY_PREFIX_BYTES, CHECKSUM_BYTES, FORMAT_VERSION,
    HEADER_BYTES, MAGIC, MAXIMUM_ENTITY_NAME_BYTES, MAXIMUM_PRODUCER_IDENTITY_BYTES,
    MAXIMUM_ROLE_IDENTITY_BYTES, MINIMUM_ACCEPTED_OUTPUT_BYTES, WORKFLOW_FACTS, WORKFLOW_INSTANCES,
};
