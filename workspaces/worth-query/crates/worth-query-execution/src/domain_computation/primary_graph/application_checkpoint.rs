use sha2::{Digest, Sha256};

mod capture;
mod encode;
mod facts;
mod resources;
mod section_bytes;
#[cfg(test)]
mod tests;
#[cfg(test)]
use capture::merge_accepted_outputs;
pub(in crate::domain_computation::primary_graph) use facts::decode as decode_producer_facts;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use facts::encode as encode_producer_facts;
pub use section_bytes::{
    WorthQueryApplicationCheckpointSectionBytes, WorthQueryNativeCheckpointSectionBytes,
};

const MAGIC: &[u8; 8] = b"WQAPCP01";
const FORMAT_VERSION: u16 = 5;
const CHECKSUM_BYTES: usize = 32;
const BODY_PREFIX_BYTES: usize = 2 + 8 + 8 + 8;
const HEADER_BYTES: usize = MAGIC.len() + CHECKSUM_BYTES + BODY_PREFIX_BYTES;
const LEGACY_MINIMUM_ACCEPTED_OUTPUT_BYTES: usize = 8 + 1 + 32 + 16 + 32 + 33 + 32 + 8;
const MINIMUM_ACCEPTED_OUTPUT_BYTES: usize = LEGACY_MINIMUM_ACCEPTED_OUTPUT_BYTES + 17;
const MINIMUM_V5_ACCEPTED_OUTPUT_BYTES: usize = MINIMUM_ACCEPTED_OUTPUT_BYTES + 8;
const MAXIMUM_PRODUCER_IDENTITY_BYTES: usize = 4 * 1024;
const MAXIMUM_ROLE_IDENTITY_BYTES: usize = 4 * 1024;
const MAXIMUM_ENTITY_NAME_BYTES: usize = 4 * 1024;

/// Opaque bytes for one complete Query application checkpoint.
///
/// Query alone defines and admits the payload. Hosts retain and transport the
/// bytes without interpreting model facts or reconstructing a partial runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCheckpoint {
    bytes: Box<[u8]>,
}

pub(in crate::domain_computation::primary_graph) struct DecodedApplicationCheckpoint {
    pub(super) native: worth_relational::facade::durability::RelationalNativeCheckpoint,
    pub(super) bootstrap_commit_id: worth_relational::facade::history::CommitId,
    pub(super) accepted_outputs:
        Vec<super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity>,
}

impl WorthQueryApplicationCheckpoint {
    pub fn from_untrusted_bytes(bytes: impl Into<Box<[u8]>>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[cfg(feature = "test-durability-faults")]
    #[doc(hidden)]
    pub fn rewrite_authenticated_body_for_durability_test(
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

    pub(super) fn decode(self) -> Result<DecodedApplicationCheckpoint, String> {
        if self.bytes.len() < HEADER_BYTES || &self.bytes[..MAGIC.len()] != MAGIC {
            return Err("Query application checkpoint header is invalid".to_owned());
        }
        let checksum_start = MAGIC.len();
        let body_start = checksum_start + CHECKSUM_BYTES;
        let expected_checksum = &self.bytes[checksum_start..body_start];
        let body = &self.bytes[body_start..];
        if Sha256::digest(body).as_slice() != expected_checksum {
            return Err("Query application checkpoint checksum differs".to_owned());
        }
        let version = u16::from_be_bytes([body[0], body[1]]);
        if version != FORMAT_VERSION && version != 4 && version != 3 {
            return Err(format!(
                "Query application checkpoint format {version} is unsupported"
            ));
        }
        let mut cursor = CheckpointCursor::new(&body[2..]);
        let bootstrap_commit_id = worth_relational::facade::history::CommitId(cursor.next_u64()?);
        let native_len = usize::try_from(cursor.next_u64()?)
            .map_err(|_| "checkpoint payload length exceeds this host".to_owned())?;
        let accepted_count = usize::try_from(cursor.next_u64()?)
            .map_err(|_| "checkpoint accepted-output count exceeds this host".to_owned())?;
        cursor.next_bytes(native_len)?;
        let minimum = match version {
            3 => LEGACY_MINIMUM_ACCEPTED_OUTPUT_BYTES,
            4 => MINIMUM_ACCEPTED_OUTPUT_BYTES,
            _ => MINIMUM_V5_ACCEPTED_OUTPUT_BYTES,
        };
        if accepted_count > cursor.remaining.len() / minimum {
            return Err("checkpoint accepted-output count exceeds its payload".to_owned());
        }
        let mut accepted_outputs = Vec::with_capacity(accepted_count);
        for _ in 0..accepted_count {
            let producer_len = usize::try_from(cursor.next_u64()?)
                .map_err(|_| "checkpoint producer identity length exceeds this host".to_owned())?;
            if producer_len == 0 || producer_len > MAXIMUM_PRODUCER_IDENTITY_BYTES {
                return Err("checkpoint producer identity length is invalid".to_owned());
            }
            let producer = std::str::from_utf8(cursor.next_bytes(producer_len)?)
                .map_err(|_| "checkpoint producer identity is not UTF-8".to_owned())?
                .to_owned();
            let source = cursor
                .next_bytes(32)?
                .try_into()
                .expect("the checkpoint source identity length is exact");
            let scope = cursor.next_scope()?;
            let source_partition = cursor
                .next_bytes(32)?
                .try_into()
                .expect("the checkpoint source-partition identity length is exact");
            let producer_dependency = match cursor.next_byte()? {
                0 => {
                    let empty = cursor.next_bytes(32)?;
                    if empty.iter().any(|byte| *byte != 0) {
                        return Err(
                            "checkpoint absent producer dependency contains data".to_owned()
                        );
                    }
                    None
                }
                1 => Some(
                    cursor
                        .next_bytes(32)?
                        .try_into()
                        .expect("the checkpoint producer dependency length is exact"),
                ),
                _ => return Err("checkpoint producer dependency posture is invalid".to_owned()),
            };
            let idempotency_key = cursor
                .next_bytes(32)?
                .try_into()
                .expect("the checkpoint idempotency identity length is exact");
            let resources = resources::decode_profile(&mut cursor, version)?;
            let role_count = usize::try_from(cursor.next_u64()?)
                .map_err(|_| "checkpoint output-role count exceeds this host".to_owned())?;
            if role_count > cursor.remaining.len() / (8 + 1 + 8 + 16) {
                return Err("checkpoint output-role count exceeds its payload".to_owned());
            }
            let mut roles = Vec::with_capacity(role_count);
            for _ in 0..role_count {
                let role = cursor.next_bounded_text(
                    MAXIMUM_ROLE_IDENTITY_BYTES,
                    "checkpoint output-role identity",
                )?;
                let posture = match cursor.next_byte()? {
                    0 => super::WorthQueryApplicationOutputPosture::Preserve,
                    1 => super::WorthQueryApplicationOutputPosture::Create,
                    2 => super::WorthQueryApplicationOutputPosture::Retire,
                    _ => return Err("checkpoint output-role posture is invalid".to_owned()),
                };
                let entity_name = cursor.next_bounded_text(
                    MAXIMUM_ENTITY_NAME_BYTES,
                    "checkpoint output-role entity name",
                )?;
                roles.push(super::application_attempt::WorthQueryCheckpointOutputRole {
                    role,
                    posture,
                    entity_name,
                    entity: cursor.next_entity()?,
                });
            }
            if roles.windows(2).any(|pair| pair[0].role >= pair[1].role) {
                return Err("checkpoint output roles are duplicated or non-canonical".to_owned());
            }
            let producer_facts = if version >= 5 {
                let fact_len = usize::try_from(cursor.next_u64()?)
                    .map_err(|_| "checkpoint producer fact length exceeds this host".to_owned())?;
                if fact_len > facts::MAXIMUM_FACT_BYTES {
                    return Err("checkpoint producer fact payload length is invalid".to_owned());
                }
                if fact_len == 0 {
                    None
                } else {
                    let bytes = cursor.next_bytes(fact_len)?;
                    facts::decode(bytes)?;
                    Some(bytes.to_vec())
                }
            } else {
                None
            };
            accepted_outputs.push(
                super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity {
                    producer,
                    source,
                    scope,
                    source_partition,
                    producer_dependency,
                    idempotency_key,
                    resources,
                    roles,
                    producer_facts,
                },
            );
        }
        if accepted_outputs
            .windows(2)
            .any(|pair| pair[0].canonical_cmp(&pair[1]) != std::cmp::Ordering::Less)
        {
            return Err("checkpoint accepted outputs are duplicated or non-canonical".to_owned());
        }
        let mut slots = std::collections::BTreeSet::new();
        if accepted_outputs.iter().any(|accepted| {
            !slots.insert((
                accepted.producer.as_str(),
                accepted.scope,
                accepted.source_partition,
            ))
        }) {
            return Err("checkpoint output slots are duplicated".to_owned());
        }
        if !cursor.is_empty() {
            return Err("Query application checkpoint payload length differs".to_owned());
        }
        let native_end = HEADER_BYTES
            .checked_add(native_len)
            .ok_or_else(|| "checkpoint native payload length overflows".to_owned())?;
        let native = worth_relational::facade::durability::RelationalNativeCheckpoint::from_untrusted_bytes_region(
            self.bytes,
            HEADER_BYTES..native_end,
        )
        .map_err(str::to_owned)?;
        Ok(DecodedApplicationCheckpoint {
            native,
            bootstrap_commit_id,
            accepted_outputs,
        })
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
