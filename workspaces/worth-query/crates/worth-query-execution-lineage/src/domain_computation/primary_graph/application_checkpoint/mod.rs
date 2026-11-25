use worth_query_declaration::archive_decode_authority::{ArchiveDecodeAuthority, ARCHIVE_DECODE};
pub mod capture {
    pub mod output_facts;
}
pub mod accepted_output;
pub mod facts;
pub mod header;
pub mod metadata_funding;
pub mod producer_computation;

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

pub use facts::FactDecodeDenial;
pub use header::CheckpointDecodeDenial;
pub use metadata_funding::CheckpointMetadataFunding;
pub use producer_computation::{decode as decode_checkpoint_computation, CheckpointProducerFacts};

pub const MAXIMUM_PRODUCER_IDENTITY_BYTES: usize = 4 * 1024;
pub const MAXIMUM_ROLE_IDENTITY_BYTES: usize = 4 * 1024;
pub const MAXIMUM_ENTITY_NAME_BYTES: usize = 4 * 1024;
