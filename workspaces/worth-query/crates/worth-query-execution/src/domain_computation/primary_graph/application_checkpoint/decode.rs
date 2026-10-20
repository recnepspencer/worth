//! Current checkpoint metadata reconstruction under the bootstrap payer.
use super::*;

impl WorthQueryApplicationCheckpoint {
    pub(super) fn decode_current_body(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial> {
        let body_start = MAGIC.len() + CHECKSUM_BYTES;
        let body = &self.bytes[body_start..];
        let mut cursor = CheckpointCursor::new(&body[6..]);
        let bootstrap_commit_id = worth_relational::facade::history::CommitId(cursor.next_u64()?);
        let native_len = usize::try_from(cursor.next_u64()?)
            .map_err(|_| "checkpoint payload length exceeds this host".to_owned())?;
        let accepted_count = usize::try_from(cursor.next_u64()?)
            .map_err(|_| "checkpoint accepted-output count exceeds this host".to_owned())?;
        cursor.next_bytes(native_len)?;
        let minimum = MINIMUM_ACCEPTED_OUTPUT_BYTES;
        if accepted_count > cursor.remaining.len() / minimum {
            return Err("checkpoint accepted-output count exceeds its payload"
                .to_owned()
                .into());
        }
        let mut metadata = CheckpointMetadataFunding::new(execution)?;
        let mut accepted_outputs = metadata.vec::<super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity>(accepted_count, execution)?;
        for _ in 0..accepted_count {
            let mut identity_funding = CheckpointMetadataFunding::new(execution)?;
            identity_funding.check_live(execution)?;
            let producer_len = usize::try_from(cursor.next_u64()?)
                .map_err(|_| "checkpoint producer identity length exceeds this host".to_owned())?;
            if producer_len == 0 || producer_len > MAXIMUM_PRODUCER_IDENTITY_BYTES {
                return Err("checkpoint producer identity length is invalid"
                    .to_owned()
                    .into());
            }
            let producer = std::str::from_utf8(cursor.next_bytes(producer_len)?)
                .map_err(|_| "checkpoint producer identity is not UTF-8".to_owned())?;
            let producer = identity_funding.copy_text(producer, execution)?;
            let posture = match cursor.next_byte()? {
                0 => super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture::Performed,
                1 => super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture::StableReused,
                _ => return Err("checkpoint accepted output posture is invalid".to_owned().into()),
            };
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
                        return Err("checkpoint absent producer dependency contains data"
                            .to_owned()
                            .into());
                    }
                    None
                }
                1 => Some(
                    cursor
                        .next_bytes(32)?
                        .try_into()
                        .expect("the checkpoint producer dependency length is exact"),
                ),
                _ => {
                    return Err("checkpoint producer dependency posture is invalid"
                        .to_owned()
                        .into())
                }
            };
            let idempotency_key = cursor
                .next_bytes(32)?
                .try_into()
                .expect("the checkpoint idempotency identity length is exact");
            let role_count = usize::try_from(cursor.next_u64()?)
                .map_err(|_| "checkpoint output-role count exceeds this host".to_owned())?;
            if role_count > cursor.remaining.len() / (8 + 1 + 8 + 16) {
                return Err("checkpoint output-role count exceeds its payload"
                    .to_owned()
                    .into());
            }
            let mut roles = identity_funding
                .vec::<super::super::application_attempt::WorthQueryCheckpointOutputRole>(
                role_count, execution,
            )?;
            for _ in 0..role_count {
                let role = identity_funding.text(
                    &mut cursor,
                    execution,
                    MAXIMUM_ROLE_IDENTITY_BYTES,
                    "checkpoint output-role identity",
                )?;
                let posture = match cursor.next_byte()? {
                    0 => super::super::WorthQueryApplicationOutputPosture::Preserve,
                    1 => super::super::WorthQueryApplicationOutputPosture::Create,
                    2 => super::super::WorthQueryApplicationOutputPosture::Retire,
                    _ => {
                        return Err("checkpoint output-role posture is invalid"
                            .to_owned()
                            .into())
                    }
                };
                let entity_name = identity_funding.text(
                    &mut cursor,
                    execution,
                    MAXIMUM_ENTITY_NAME_BYTES,
                    "checkpoint output-role entity name",
                )?;
                roles.push(
                    super::super::application_attempt::WorthQueryCheckpointOutputRole {
                        role,
                        posture,
                        entity_name,
                        entity: cursor.next_entity()?,
                    },
                );
            }
            if roles.windows(2).any(|pair| pair[0].role >= pair[1].role) {
                return Err("checkpoint output roles are duplicated or non-canonical"
                    .to_owned()
                    .into());
            }
            let fact_version = cursor.next_u16()?;
            let fact_len = usize::try_from(cursor.next_u64()?)
                .map_err(|_| "checkpoint producer fact length exceeds this host".to_owned())?;
            if fact_len != 0 && fact_version != facts::WIRE_VERSION {
                return Err(CheckpointDecodeDenial::unsupported_producer_facts(
                    fact_version,
                ));
            }
            if fact_len > facts::MAXIMUM_FACT_BYTES {
                return Err("checkpoint producer fact payload length is invalid"
                    .to_owned()
                    .into());
            }
            let (producer_facts, producer_fact_wire_version) = if fact_len == 0 {
                if fact_version != 0 {
                    return Err("checkpoint empty producer facts carry a wire version"
                        .to_owned()
                        .into());
                }
                (None, 0)
            } else {
                let bytes = cursor.next_bytes(fact_len)?;
                // Bootstrap has no authenticated Query request. The actual
                // execution policy/control still reaches the one current parser.
                let mut decoder = facts::FactDecoder::new(
                    bytes,
                    fact_version,
                    None,
                    execution.allocation_policy(),
                )
                .map_err(CheckpointDecodeDenial::from_fact)?;
                while decoder
                    .next_fact()
                    .map_err(CheckpointDecodeDenial::from_fact)?
                    .is_some()
                {
                    identity_funding.check_live(execution)?;
                }
                // The existing retained checkpoint identity owns Vec<u8>, so
                // this required retained copy is admitted before it is made.
                let mut retained = identity_funding.vec::<u8>(bytes.len(), execution)?;
                retained.extend_from_slice(bytes);
                (Some(retained), fact_version)
            };
            identity_funding.slots::<String>(1, execution)?;
            identity_funding.slots::<Vec<
                super::super::application_attempt::WorthQueryCheckpointOutputRole,
            >>(1, execution)?;
            if producer_facts.is_some() {
                identity_funding.slots::<Vec<u8>>(1, execution)?;
            }
            identity_funding.slots::<CheckpointMetadataFunding>(1, execution)?;
            accepted_outputs.push(
                super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity {
                    producer: std::sync::Arc::new(producer),
                    posture,
                    source,
                    scope,
                    source_partition,
                    producer_dependency,
                    idempotency_key,
                    roles: std::sync::Arc::new(roles),
                    producer_facts: producer_facts.map(std::sync::Arc::new),
                    producer_fact_wire_version,
                    metadata: Some(std::sync::Arc::new(identity_funding)),
                },
            );
        }
        if accepted_outputs
            .windows(2)
            .any(|pair| pair[0].canonical_cmp(&pair[1]) != std::cmp::Ordering::Less)
        {
            return Err(
                "checkpoint accepted outputs are duplicated or non-canonical"
                    .to_owned()
                    .into(),
            );
        }
        // A funded temporary permutation checks duplicate slots without copying
        // producer strings into another authoritative inventory.
        let mut scratch = CheckpointMetadataFunding::new(execution)?;
        let mut slots = scratch.vec::<usize>(accepted_count, execution)?;
        slots.extend(0..accepted_count);
        slots.sort_unstable_by(|left, right| {
            (
                &accepted_outputs[*left].producer,
                accepted_outputs[*left].scope,
                accepted_outputs[*left].source_partition,
            )
                .cmp(&(
                    &accepted_outputs[*right].producer,
                    accepted_outputs[*right].scope,
                    accepted_outputs[*right].source_partition,
                ))
        });
        if slots.windows(2).any(|pair| {
            accepted_outputs[pair[0]].output_slot() == accepted_outputs[pair[1]].output_slot()
        }) {
            return Err("checkpoint output slots are duplicated".to_owned().into());
        }
        drop(slots);
        drop(scratch);
        if !cursor.is_empty() {
            return Err("Query application checkpoint payload length differs"
                .to_owned()
                .into());
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
            metadata,
        })
    }
}
