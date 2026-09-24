use sha2::{Digest, Sha256};

mod resources;
#[cfg(test)]
mod tests;

const MAGIC: &[u8; 8] = b"WQAPCP01";
const FORMAT_VERSION: u16 = 4;
const CHECKSUM_BYTES: usize = 32;
const BODY_PREFIX_BYTES: usize = 2 + 8 + 8 + 8;
const HEADER_BYTES: usize = MAGIC.len() + CHECKSUM_BYTES + BODY_PREFIX_BYTES;
const LEGACY_MINIMUM_ACCEPTED_OUTPUT_BYTES: usize = 8 + 1 + 32 + 16 + 32 + 33 + 32 + 8;
const MINIMUM_ACCEPTED_OUTPUT_BYTES: usize = LEGACY_MINIMUM_ACCEPTED_OUTPUT_BYTES + 17;
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

    fn encode(
        native: worth_relational::facade::durability::RelationalNativeCheckpoint,
        publication: &super::WorthQueryPrimaryGraphPublication,
        accepted_outputs: &[super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity],
    ) -> Self {
        let native_bytes = native.bytes();
        let accepted_bytes = accepted_outputs.iter().fold(0_usize, |total, accepted| {
            let role_bytes = accepted.roles.iter().fold(0_usize, |role_total, role| {
                role_total.saturating_add(8 + role.role.len() + 1 + 8 + role.entity_name.len() + 16)
            });
            total.saturating_add(
                MINIMUM_ACCEPTED_OUTPUT_BYTES - 1 + accepted.producer.len() + role_bytes,
            )
        });
        let mut body = Vec::with_capacity(BODY_PREFIX_BYTES + native_bytes.len() + accepted_bytes);
        body.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
        body.extend_from_slice(&publication.bootstrap_commit_id().0.to_be_bytes());
        body.extend_from_slice(&(native_bytes.len() as u64).to_be_bytes());
        body.extend_from_slice(&(accepted_outputs.len() as u64).to_be_bytes());
        body.extend_from_slice(native_bytes);
        for accepted in accepted_outputs {
            body.extend_from_slice(&(accepted.producer.len() as u64).to_be_bytes());
            body.extend_from_slice(accepted.producer.as_bytes());
            body.extend_from_slice(&accepted.source);
            encode_scope(&mut body, accepted.scope);
            body.extend_from_slice(&accepted.source_partition);
            body.push(u8::from(accepted.producer_dependency.is_some()));
            body.extend_from_slice(&accepted.producer_dependency.unwrap_or_default());
            body.extend_from_slice(&accepted.idempotency_key);
            resources::encode_profile(&mut body, accepted.resources);
            body.extend_from_slice(&(accepted.roles.len() as u64).to_be_bytes());
            for role in &accepted.roles {
                body.extend_from_slice(&(role.role.len() as u64).to_be_bytes());
                body.extend_from_slice(role.role.as_bytes());
                body.push(match role.posture {
                    super::WorthQueryApplicationOutputPosture::Preserve => 0,
                    super::WorthQueryApplicationOutputPosture::Create => 1,
                    super::WorthQueryApplicationOutputPosture::Retire => 2,
                });
                body.extend_from_slice(&(role.entity_name.len() as u64).to_be_bytes());
                body.extend_from_slice(role.entity_name.as_bytes());
                encode_entity(&mut body, role.entity);
            }
        }
        let checksum = Sha256::digest(&body);
        let mut bytes = Vec::with_capacity(MAGIC.len() + CHECKSUM_BYTES + body.len());
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&checksum);
        bytes.extend_from_slice(&body);
        Self {
            bytes: bytes.into_boxed_slice(),
        }
    }

    pub(super) fn decode(&self) -> Result<DecodedApplicationCheckpoint, String> {
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
        if version != FORMAT_VERSION && version != 3 {
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
        let native = cursor.next_bytes(native_len)?;
        let minimum = if version == 3 {
            LEGACY_MINIMUM_ACCEPTED_OUTPUT_BYTES
        } else {
            MINIMUM_ACCEPTED_OUTPUT_BYTES
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
                },
            );
        }
        if accepted_outputs.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err("checkpoint accepted outputs are duplicated or non-canonical".to_owned());
        }
        if !cursor.is_empty() {
            return Err("Query application checkpoint payload length differs".to_owned());
        }
        Ok(DecodedApplicationCheckpoint {
            native: worth_relational::facade::durability::RelationalNativeCheckpoint::from_untrusted_bytes(
                native.to_vec().into_boxed_slice(),
            ),
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

fn encode_scope(
    output: &mut Vec<u8>,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
) {
    output.extend_from_slice(&scope.partition_id().to_be_bytes());
    output.extend_from_slice(&scope.local_slot().to_be_bytes());
    output.extend_from_slice(&scope.generation().to_be_bytes());
}

fn encode_entity(output: &mut Vec<u8>, entity: worth_relational::facade::identity::EntityId) {
    output.extend_from_slice(&entity.partition_value().to_be_bytes());
    output.extend_from_slice(&entity.local_slot_value().to_be_bytes());
    output.extend_from_slice(&entity.generation_value().to_be_bytes());
}

fn merge_accepted_outputs(
    mut current: Vec<super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity>,
    recovered: impl IntoIterator<
        Item = super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
    >,
) -> Vec<super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity> {
    let unshadowed = recovered
        .into_iter()
        .filter(|recovered| {
            !current
                .iter()
                .any(|accepted| accepted.same_output_slot(recovered))
        })
        .collect::<Vec<_>>();
    current.extend(unshadowed);
    current.sort();
    current.dedup();
    current
}

impl<Schema> super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static,
{
    pub fn capture_application_checkpoint(
        &self,
    ) -> Result<
        WorthQueryApplicationCheckpoint,
        worth_relational::facade::durability::DurabilityError,
    > {
        self.primary_provider.graph.with_runtime(|runtime| {
            runtime
                .durability_authority()
                .native_checkpoint()
                .map(|checkpoint| {
                    let accepted_outputs = merge_accepted_outputs(
                        self.output_demands.accepted_checkpoint_identities(),
                        self.recovered_outputs
                            .iter()
                            .map(|accepted| accepted.checkpoint.clone()),
                    );
                    WorthQueryApplicationCheckpoint::encode(
                        checkpoint,
                        self.publication(),
                        &accepted_outputs,
                    )
                })
        })
    }
}
