//! The existing accepted-output format is shared by checkpoint and obligation custody.
use super::super::application_output_demand::{
    WorthQueryAcceptedOutputCheckpointIdentity, WorthQueryAcceptedOutputCheckpointPosture,
};
use super::super::WorthQueryApplicationOutputPosture;
use super::*;

pub fn encode(
    accepted: &WorthQueryAcceptedOutputCheckpointIdentity,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<Vec<u8>, String> {
    policy
        .check_live()
        .map_err(|cause| format!("accepted output allocation policy stopped: {cause:?}"))?;
    let facts = accepted
        .producer_facts
        .as_ref()
        .map_or(&[][..], |bytes| bytes.as_slice());
    let mut total = 8usize
        .checked_add(accepted.producer.len())
        .and_then(|n| n.checked_add(1 + 16 + 32 + 32 + 16 + 32 + 1 + 32 + 32 + 8))
        .ok_or_else(|| "accepted output size overflow".to_owned())?;
    for role in accepted.roles.iter() {
        total = total
            .checked_add(8)
            .and_then(|n| n.checked_add(role.role.len()))
            .and_then(|n| n.checked_add(1 + 8))
            .and_then(|n| n.checked_add(role.entity_name.len()))
            .and_then(|n| n.checked_add(16))
            .ok_or_else(|| "accepted output role size overflow".to_owned())?;
    }
    total = total
        .checked_add(2 + 8)
        .and_then(|n| n.checked_add(facts.len()))
        .ok_or_else(|| "accepted output facts size overflow".to_owned())?;
    let mut bytes = worth_execution::ExecutionByteBuffer::allocate(total, policy)
        .map_err(|cause| format!("accepted output allocation failed: {cause:?}"))?;
    let push_len = |buf: &mut worth_execution::ExecutionByteBuffer, len: usize| -> Result<(), String> {
        let wire = u64::try_from(len).map_err(|_| "accepted output length overflow".to_owned())?;
        buf.extend_from_slice(&wire.to_be_bytes())
            .map_err(|cause| format!("accepted output write failed: {cause:?}"))
    };
    let push_bytes =
        |buf: &mut worth_execution::ExecutionByteBuffer, slice: &[u8]| -> Result<(), String> {
            buf.extend_from_slice(slice)
                .map_err(|cause| format!("accepted output write failed: {cause:?}"))
        };
    push_len(&mut bytes, accepted.producer.len())?;
    push_bytes(&mut bytes, accepted.producer.as_bytes())?;
    push_bytes(
        &mut bytes,
        &[match accepted.posture {
            WorthQueryAcceptedOutputCheckpointPosture::Performed => 0,
            WorthQueryAcceptedOutputCheckpointPosture::StableReused => 1,
        }],
    )?;
    push_bytes(&mut bytes, &accepted.implementation.build_identity())?;
    push_bytes(&mut bytes, &accepted.implementation.edition())?;
    push_bytes(&mut bytes, &accepted.source)?;
    push_bytes(&mut bytes, &accepted.scope.partition_id().to_be_bytes())?;
    push_bytes(&mut bytes, &accepted.scope.local_slot().to_be_bytes())?;
    push_bytes(&mut bytes, &accepted.scope.generation().to_be_bytes())?;
    push_bytes(&mut bytes, &accepted.source_partition)?;
    push_bytes(&mut bytes, &[u8::from(accepted.producer_dependency.is_some())])?;
    push_bytes(&mut bytes, &accepted.producer_dependency.unwrap_or_default())?;
    push_bytes(&mut bytes, &accepted.idempotency_key)?;
    push_len(&mut bytes, accepted.roles.len())?;
    for role in accepted.roles.iter() {
        push_len(&mut bytes, role.role.len())?;
        push_bytes(&mut bytes, role.role.as_bytes())?;
        push_bytes(
            &mut bytes,
            &[match role.posture {
                WorthQueryApplicationOutputPosture::Preserve => 0,
                WorthQueryApplicationOutputPosture::Create => 1,
                WorthQueryApplicationOutputPosture::Retire => 2,
            }],
        )?;
        push_len(&mut bytes, role.entity_name.len())?;
        push_bytes(&mut bytes, role.entity_name.as_bytes())?;
        push_bytes(&mut bytes, &role.entity.partition_value().to_be_bytes())?;
        push_bytes(&mut bytes, &role.entity.local_slot_value().to_be_bytes())?;
        push_bytes(&mut bytes, &role.entity.generation_value().to_be_bytes())?;
    }
    push_bytes(&mut bytes, &accepted.producer_fact_wire_version.to_be_bytes())?;
    push_len(&mut bytes, facts.len())?;
    push_bytes(&mut bytes, facts)?;
    if bytes.len() != total {
        return Err("accepted output encoded length mismatch".to_owned());
    }
    Ok(bytes.bytes().to_vec())
}

pub fn decode(
    bytes: &[u8],
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorthQueryAcceptedOutputCheckpointIdentity, CheckpointDecodeDenial> {
    let mut cursor = CheckpointCursor::new(bytes);
    let output = read(&mut cursor, execution)?;
    if !cursor.is_empty() {
        return Err("accepted output payload length differs".to_owned().into());
    }
    Ok(output)
}
pub fn read(
    cursor: &mut CheckpointCursor<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorthQueryAcceptedOutputCheckpointIdentity, CheckpointDecodeDenial> {
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
    let implementation =
        worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint(
            cursor
                .next_bytes(16)?
                .try_into()
                .expect("exact build identity width"),
            cursor
                .next_bytes(32)?
                .try_into()
                .expect("exact producing edition width"),
        )
        .map_err(|cause| cause.to_owned())?;
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
            cursor,
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
            cursor,
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
        let mut decoder =
            facts::FactDecoder::new(bytes, fact_version, None, execution.allocation_policy())
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
    identity_funding
        .slots::<Vec<super::super::application_attempt::WorthQueryCheckpointOutputRole>>(
            1, execution,
        )?;
    if producer_facts.is_some() {
        identity_funding.slots::<Vec<u8>>(1, execution)?;
    }
    identity_funding.slots::<CheckpointMetadataFunding>(1, execution)?;
    Ok(
        super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity {
            implementation,
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
    )
}
