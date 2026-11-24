use sha2::{Digest, Sha256};

use crate::aspect_wire::{encode_string, encode_u32};
use crate::history::data::coordinates::BranchId;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::symbols::data::ClientKey;

use crate::transactions::data::primitives::{
    EntityReference, PlannedLineageTransition, TransactionId,
};

pub fn bulk_naming_plan_digest(normalized_client_keys: &[ClientKey]) -> String {
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "transaction.bulk.naming.v1");
    encode_client_keys(&mut bytes, normalized_client_keys);
    sha256_hex(&bytes)
}

pub fn bulk_lineage_plan_digest(transitions: &[PlannedLineageTransition]) -> String {
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "transaction.bulk.lineage.v1");
    encode_usize(&mut bytes, transitions.len());
    for transition in transitions {
        encode_lineage_transition(&mut bytes, transition);
    }
    sha256_hex(&bytes)
}

pub fn bulk_provenance_plan_digest(
    transaction_id: TransactionId,
    target_branch: Option<&BranchId>,
    batch_name: &str,
    worker_batch_names: &[String],
    worker_partition_keys: &[Option<String>],
    worker_local_only_flags: &[bool],
) -> String {
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "transaction.bulk.provenance.v1");
    encode_u64(&mut bytes, transaction_id.0);
    encode_optional_branch_id(&mut bytes, target_branch);
    encode_string(&mut bytes, batch_name);
    encode_strings(&mut bytes, worker_batch_names);
    encode_optional_strings(&mut bytes, worker_partition_keys);
    encode_bools(&mut bytes, worker_local_only_flags);
    sha256_hex(&bytes)
}

fn encode_lineage_transition(bytes: &mut Vec<u8>, transition: &PlannedLineageTransition) {
    match transition {
        PlannedLineageTransition::CreateEntity {
            partition_id,
            kind_id,
            client_key,
        } => {
            bytes.push(0);
            encode_partition_id(bytes, *partition_id);
            encode_kind_id(bytes, *kind_id);
            encode_client_key(bytes, client_key);
        }
        PlannedLineageTransition::ReplaceEntity {
            entity_id,
            replacement_partition_id,
            replacement_kind_id,
            replacement_client_key,
        } => {
            bytes.push(1);
            encode_entity_id(bytes, *entity_id);
            encode_partition_id(bytes, *replacement_partition_id);
            encode_kind_id(bytes, *replacement_kind_id);
            encode_client_key(bytes, replacement_client_key);
        }
        PlannedLineageTransition::DeleteEntity { entity_id } => {
            bytes.push(2);
            encode_entity_id(bytes, *entity_id);
        }
        PlannedLineageTransition::CreateRelation {
            partition_id,
            kind_id,
            source,
            target,
            client_key,
        } => {
            bytes.push(3);
            encode_partition_id(bytes, *partition_id);
            encode_kind_id(bytes, *kind_id);
            encode_entity_reference(bytes, source);
            encode_entity_reference(bytes, target);
            encode_client_key(bytes, client_key);
        }
        PlannedLineageTransition::UpdateRelationEndpoints {
            relation_id,
            source,
            target,
        } => {
            bytes.push(4);
            encode_relation_id(bytes, *relation_id);
            encode_entity_reference(bytes, source);
            encode_entity_reference(bytes, target);
        }
        PlannedLineageTransition::DeleteRelation { relation_id } => {
            bytes.push(5);
            encode_relation_id(bytes, *relation_id);
        }
    }
}

fn encode_entity_reference(bytes: &mut Vec<u8>, reference: &EntityReference) {
    match reference {
        EntityReference::Existing(entity_id) => {
            bytes.push(0);
            encode_entity_id(bytes, *entity_id);
        }
        EntityReference::Created(created) => {
            bytes.push(1);
            encode_partition_id(bytes, created.partition_id);
            encode_kind_id(bytes, created.kind_id);
            encode_client_key(bytes, &created.client_key);
        }
    }
}

fn encode_client_keys(bytes: &mut Vec<u8>, client_keys: &[ClientKey]) {
    encode_usize(bytes, client_keys.len());
    for client_key in client_keys {
        encode_client_key(bytes, client_key);
    }
}

fn encode_client_key(bytes: &mut Vec<u8>, client_key: &ClientKey) {
    match (client_key.as_raw_str(), client_key.as_symbol()) {
        (Some(raw), None) => {
            bytes.push(0);
            encode_string(bytes, raw);
        }
        (None, Some(symbol)) => {
            bytes.push(1);
            encode_u32(bytes, symbol.0);
        }
        _ => {
            bytes.push(2);
            encode_string(bytes, client_key.canonical_text().as_ref());
        }
    }
}

fn encode_strings(bytes: &mut Vec<u8>, values: &[String]) {
    encode_usize(bytes, values.len());
    for value in values {
        encode_string(bytes, value);
    }
}

fn encode_optional_strings(bytes: &mut Vec<u8>, values: &[Option<String>]) {
    encode_usize(bytes, values.len());
    for value in values {
        match value {
            Some(value) => {
                bytes.push(1);
                encode_string(bytes, value);
            }
            None => bytes.push(0),
        }
    }
}

fn encode_bools(bytes: &mut Vec<u8>, values: &[bool]) {
    encode_usize(bytes, values.len());
    for value in values {
        bytes.push(u8::from(*value));
    }
}

fn encode_optional_branch_id(bytes: &mut Vec<u8>, branch_id: Option<&BranchId>) {
    match branch_id {
        Some(branch_id) => {
            bytes.push(1);
            encode_string(bytes, &branch_id.0);
        }
        None => bytes.push(0),
    }
}

fn encode_entity_id(bytes: &mut Vec<u8>, id: EntityId) {
    encode_partition_id(bytes, id.partition_id);
    encode_u64(bytes, id.local_slot.0);
    encode_u32(bytes, id.generation.0);
}

fn encode_relation_id(bytes: &mut Vec<u8>, id: RelationId) {
    encode_partition_id(bytes, id.partition_id);
    encode_u64(bytes, id.local_slot.0);
    encode_u32(bytes, id.generation.0);
}

fn encode_partition_id(bytes: &mut Vec<u8>, partition_id: PartitionId) {
    encode_u32(bytes, partition_id.0);
}

fn encode_kind_id(bytes: &mut Vec<u8>, kind_id: KindId) {
    encode_u32(bytes, kind_id.0);
}

fn encode_usize(bytes: &mut Vec<u8>, value: usize) {
    encode_u64(bytes, value as u64);
}

fn encode_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
