mod inspection;
pub(in crate::domain_computation::primary_graph) mod interim_transport;
mod transition_capture;
pub use inspection::{
    CheckpointAllocatorInspection, CheckpointRecordFrontier, CheckpointRecordIdentity,
    CheckpointReservationAdvanceDenial, WorthQueryApplicationCheckpointInspection,
    WorthQueryCheckpointInspectionDenial, WorthQueryCheckpointModelBytes,
    WorthQueryCheckpointRecordClass,
};
pub use interim_transport::WorthQueryInterimCheckpointOrigin;
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub(crate) mod capture;
mod decode;
pub(crate) mod header;
pub(crate) mod metadata_funding;
pub(in crate::domain_computation::primary_graph) use header::CheckpointDecodeDenial;
pub(in crate::domain_computation::primary_graph) use metadata_funding::CheckpointMetadataFunding;
mod denial;
pub(crate) mod encode;
pub(crate) mod facts;
mod producer_computation;
pub(in crate::domain_computation::primary_graph) use facts::FactDecodeDenial;
pub(in crate::domain_computation::primary_graph) use producer_computation::{
    decode as decode_checkpoint_computation, CheckpointProducerFacts,
};
mod section_bytes;

pub use denial::WorthQueryCheckpointCaptureDenial;

pub use section_bytes::{
    WorthQueryApplicationCheckpointSectionBytes, WorthQueryNativeCheckpointSectionBytes,
};
pub use worth_execution::ExecutionAllocationPolicy as WorthQueryCheckpointCapturePolicy;

pub const MAGIC: &[u8; 8] = b"WQAPCP01";
// Current-only transport includes Native admission and World originals. Facts
// remain wire12 (presence tag11); producer quotes are absent.
pub const FORMAT_VERSION: u16 = 14;
pub const WORKFLOW_FACTS: u16 = 2;
pub const WORKFLOW_INSTANCES: u16 = 4;
pub const CHECKSUM_BYTES: usize = 32;
pub const BODY_PREFIX_BYTES: usize = 2 + 2 + 2 + 8 + 8 + 8 + 8 + 8;
pub const HEADER_BYTES: usize = MAGIC.len() + CHECKSUM_BYTES + BODY_PREFIX_BYTES;
// Version 9 preserves the row layout and certifies the exporter exclusion.
pub const MINIMUM_ACCEPTED_OUTPUT_BYTES: usize =
    8 + 1 + 48 + 32 + 16 + 32 + 33 + 32 + 8 + 8 + 1 + 2;
pub const MAXIMUM_PRODUCER_IDENTITY_BYTES: usize = 4 * 1024;
pub const MAXIMUM_ROLE_IDENTITY_BYTES: usize = 4 * 1024;
pub const MAXIMUM_ENTITY_NAME_BYTES: usize = 4 * 1024;

/// Opaque bytes for one complete Query application checkpoint.
///
/// Query alone defines and admits the payload. Hosts retain and transport the
/// bytes without interpreting model facts or reconstructing a partial runtime.
/// Clones share immutable byte backing and any actual capture-issued origin.
/// Imported bytes alone cannot grant restoration. Native and World retain
/// distinct owner readmission; neither reconstructs authority from a checksum.
#[derive(Clone, Debug)]
pub struct WorthQueryApplicationCheckpoint {
    bytes: worth_execution::ExecutionImmutableBytes,
    origin: Option<WorthQueryInterimCheckpointOrigin>,
}
impl PartialEq for WorthQueryApplicationCheckpoint {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}
impl Eq for WorthQueryApplicationCheckpoint {}

pub struct DecodedApplicationCheckpoint {
    pub(super) origin: Option<WorthQueryInterimCheckpointOrigin>,
    pub(super) native: worth_relational::facade::durability::RelationalNativeCheckpoint,
    pub(super) bootstrap_commit_id: worth_relational::facade::history::CommitId,
    pub(super) accepted_outputs:
        Vec<super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity>,
    pub(super) metadata: CheckpointMetadataFunding,
}

impl WorthQueryApplicationCheckpoint {
    pub fn from_untrusted_bytes(bytes: impl Into<Box<[u8]>>) -> Self {
        Self {
            origin: None,
            bytes: worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
                bytes.into(),
            )),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }

    /// Actual final-frame payload charge; imported and SystemAllocation bytes
    /// are uncharged. Native encoding and other capture temporaries are separate.
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        self.bytes.charged_payload_bytes()
    }

    #[doc(hidden)]
    pub fn rewrite_authenticated_checkpoint_body_for_test(
        &self,
        rewrite: impl FnOnce(&mut [u8]),
    ) -> Self {
        let mut bytes = self.bytes.to_vec();
        let body_start = MAGIC.len() + CHECKSUM_BYTES;
        rewrite(&mut bytes[body_start..]);
        let checksum = Sha256::digest(&bytes[body_start..]);
        bytes[MAGIC.len()..body_start].copy_from_slice(&checksum);
        Self::from_untrusted_bytes(bytes.into_boxed_slice())
    }

    pub(super) fn decode_with_execution(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial> {
        let (result, _) = execution
            .in_scope(|lease| {
                worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                    .run_with_checkpoint(lease, None, |context| {
                        let work = u64::try_from(self.bytes.len())
                            .map_err(|_| CheckpointDecodeDenial::capacity())?;
                        context.checkpoint_cost(work, work).map_err(|stop| {
                            CheckpointDecodeDenial::execution(
                                worth_execution::WorkCeilingDenial::Stopped(stop),
                            )
                        })?;
                        header::admit(self.bytes.bytes())?;
                        self.decode_current_body(execution)
                    })
            })
            .map_err(CheckpointDecodeDenial::execution)?
            .map_err(CheckpointDecodeDenial::execution)?;
        result
    }

    pub(super) fn decode(self) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial> {
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        self.decode_with_execution(&worth_execution::ExecutionRequest::serial(&host))
    }
}

struct CheckpointCursor<'a> {
    remaining: &'a [u8],
}

impl<'a> CheckpointCursor<'a> {
    const fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }

    fn next_u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_be_bytes(
            self.next_bytes(8)?
                .try_into()
                .expect("the checkpoint integer length is exact"),
        ))
    }

    fn next_u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_be_bytes(
            self.next_bytes(2)?
                .try_into()
                .expect("the checkpoint version length is exact"),
        ))
    }

    fn next_byte(&mut self) -> Result<u8, String> {
        Ok(self.next_bytes(1)?[0])
    }

    fn next_bounded_text(&mut self, maximum: usize, subject: &str) -> Result<String, String> {
        let length = usize::try_from(self.next_u64()?)
            .map_err(|_| format!("{subject} length exceeds this host"))?;
        if length == 0 || length > maximum {
            return Err(format!("{subject} length is invalid"));
        }
        std::str::from_utf8(self.next_bytes(length)?)
            .map(str::to_owned)
            .map_err(|_| format!("{subject} is not UTF-8"))
    }

    fn next_scope(
        &mut self,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        String,
    > {
        Ok(
            crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
                self.next_entity()?,
            ),
        )
    }

    fn next_entity(&mut self) -> Result<worth_relational::facade::identity::EntityId, String> {
        let partition = self.next_u32()?;
        let local_slot = self.next_u64()?;
        let generation = self.next_u32()?;
        Ok(worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId(partition),
            local_slot,
            generation,
        ))
    }

    fn next_u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_be_bytes(
            self.next_bytes(4)?
                .try_into()
                .expect("the checkpoint integer length is exact"),
        ))
    }

    fn next_bytes(&mut self, length: usize) -> Result<&'a [u8], String> {
        let (next, remaining) = self
            .remaining
            .split_at_checked(length)
            .ok_or_else(|| "Query application checkpoint payload is truncated".to_owned())?;
        self.remaining = remaining;
        Ok(next)
    }

    const fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}

mod court_doors_for_test;
