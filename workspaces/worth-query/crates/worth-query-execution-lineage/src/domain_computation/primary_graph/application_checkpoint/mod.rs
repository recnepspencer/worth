pub mod capture {
    pub mod output_facts;
}
pub mod interim_transport;
pub mod accepted_output;
pub mod decode;
pub mod denial;
pub mod encode;
pub mod facts;
pub mod header;
pub mod metadata_funding;
pub mod producer_computation;
pub mod section_bytes;
pub mod transition_capture;

pub use interim_transport::WorthQueryInterimCheckpointOrigin;
pub use sha2::{Digest, Sha256};
pub use worth_execution::ExecutionAllocationPolicy as WorthQueryCheckpointCapturePolicy;

#[derive(Clone, Debug)]
pub struct WorthQueryApplicationCheckpoint {
    pub bytes: worth_execution::ExecutionImmutableBytes,
    pub origin: Option<WorthQueryInterimCheckpointOrigin>,
}

impl PartialEq for WorthQueryApplicationCheckpoint {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}
impl Eq for WorthQueryApplicationCheckpoint {}

pub struct DecodedApplicationCheckpoint {
    pub origin: Option<WorthQueryInterimCheckpointOrigin>,
    pub native: worth_relational::facade::durability::RelationalNativeCheckpoint,
    pub bootstrap_commit_id: worth_relational::facade::history::CommitId,
    pub accepted_outputs:
        Vec<super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity>,
    pub metadata: CheckpointMetadataFunding,
}

impl WorthQueryApplicationCheckpoint {
    pub fn from_untrusted_bytes(bytes: impl Into<Box<[u8]>>) -> Self {
        Self {
            origin: None,
            bytes: worth_execution::ExecutionImmutableBytes::from_external_bytes(
                std::sync::Arc::new(bytes.into()),
            ),
        }
    }

    pub fn from_admitted_parts(
        bytes: worth_execution::ExecutionImmutableBytes,
        origin: Option<WorthQueryInterimCheckpointOrigin>,
    ) -> Self {
        Self { bytes, origin }
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }

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

    pub fn decode_with_execution(
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

    pub fn decode(self) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial> {
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        self.decode_with_execution(&worth_execution::ExecutionRequest::serial(&host))
    }
}

pub struct CheckpointCursor<'a> {
    remaining: &'a [u8],
}

impl<'a> CheckpointCursor<'a> {
    pub const fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }

    pub fn next_u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_be_bytes(
            self.next_bytes(8)?
                .try_into()
                .expect("the checkpoint integer length is exact"),
        ))
    }

    pub fn next_u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_be_bytes(
            self.next_bytes(2)?
                .try_into()
                .expect("the checkpoint version length is exact"),
        ))
    }

    pub fn next_byte(&mut self) -> Result<u8, String> {
        Ok(self.next_bytes(1)?[0])
    }

    pub fn next_bounded_text(&mut self, maximum: usize, subject: &str) -> Result<String, String> {
        let length = usize::try_from(self.next_u64()?)
            .map_err(|_| format!("{subject} length exceeds this host"))?;
        if length == 0 || length > maximum {
            return Err(format!("{subject} length is invalid"));
        }
        std::str::from_utf8(self.next_bytes(length)?)
            .map(str::to_owned)
            .map_err(|_| format!("{subject} is not UTF-8"))
    }

    pub fn next_scope(
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

    pub fn next_entity(&mut self) -> Result<worth_relational::facade::identity::EntityId, String> {
        let partition = self.next_u32()?;
        let local_slot = self.next_u64()?;
        let generation = self.next_u32()?;
        Ok(worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId(partition),
            local_slot,
            generation,
        ))
    }

    pub fn next_u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_be_bytes(
            self.next_bytes(4)?
                .try_into()
                .expect("the checkpoint integer length is exact"),
        ))
    }

    pub fn next_bytes(&mut self, length: usize) -> Result<&'a [u8], String> {
        let (next, remaining) = self
            .remaining
            .split_at_checked(length)
            .ok_or_else(|| "Query application checkpoint payload is truncated".to_owned())?;
        self.remaining = remaining;
        Ok(next)
    }

    pub const fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}

pub use denial::WorthQueryCheckpointCaptureDenial;
pub use facts::FactDecodeDenial;
pub use header::CheckpointDecodeDenial;
pub use metadata_funding::CheckpointMetadataFunding;
pub use producer_computation::{
    decode as decode_checkpoint_computation, CheckpointProducerFacts,
};
pub use section_bytes::{
    WorthQueryApplicationCheckpointSectionBytes, WorthQueryNativeCheckpointSectionBytes,
};

pub const MAGIC: &[u8; 8] = b"WQAPCP01";
pub const FORMAT_VERSION: u16 = 14;
pub const WORKFLOW_FACTS: u16 = 2;
pub const WORKFLOW_INSTANCES: u16 = 4;
pub const CHECKSUM_BYTES: usize = 32;
pub const BODY_PREFIX_BYTES: usize = 2 + 2 + 2 + 8 + 8 + 8 + 8 + 8;
pub const HEADER_BYTES: usize = MAGIC.len() + CHECKSUM_BYTES + BODY_PREFIX_BYTES;
pub const MINIMUM_ACCEPTED_OUTPUT_BYTES: usize =
    8 + 1 + 48 + 32 + 16 + 32 + 33 + 32 + 8 + 8 + 1 + 2;
pub const MAXIMUM_PRODUCER_IDENTITY_BYTES: usize = 4 * 1024;
pub const MAXIMUM_ROLE_IDENTITY_BYTES: usize = 4 * 1024;
pub const MAXIMUM_ENTITY_NAME_BYTES: usize = 4 * 1024;

mod court_doors_for_test;
