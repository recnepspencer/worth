mod checked;
mod primitive_terms;
mod read_record_terms;
mod scope_terms;

pub(crate) use checked::{
    entity_record_bytes, entity_record_digest_checked, query_result_reduction_digest_checked,
    relation_record_bytes, relation_record_digest_checked,
};

use sha2::{Digest, Sha256};

use crate::query::data::{
    CanonicalQueryResult, DeterministicQueryFragmentKey, DeterministicQueryPlanKey,
    IndexParityMode, QueryAccessContract, QueryAccessPath, QueryExecutionShape, QueryLocalityClass,
    QueryOrderingContract, QueryPlanContextId, QueryScope, ReductionDiscipline,
};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};

use primitive_terms::{encode_string, encode_u128, encode_u64};
use read_record_terms::{
    encode_authoritative_entity_records, encode_authoritative_relation_records,
    encode_entity_read_record, encode_relation_read_record,
};
use scope_terms::{
    encode_index_parity_mode, encode_query_access_contract, encode_query_access_path,
    encode_query_execution_shape, encode_query_locality_class, encode_query_ordering_contract,
    encode_query_plan_context_id, encode_query_scope, encode_reduction_discipline,
};

pub(crate) fn deterministic_query_plan_key_from_canonical_bytes(
    context_id: &QueryPlanContextId,
    label: &str,
    scope: &QueryScope,
    locality: &QueryLocalityClass,
    ordering: QueryOrderingContract,
    access_contract: QueryAccessContract,
    execution_shape: QueryExecutionShape,
    reduction: ReductionDiscipline,
    target_count_hint: usize,
) -> DeterministicQueryPlanKey {
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "query.plan-key.v1");
    encode_query_plan_context_id(&mut bytes, context_id);
    encode_string(&mut bytes, label);
    encode_query_scope(&mut bytes, scope);
    encode_query_locality_class(&mut bytes, locality);
    encode_query_ordering_contract(&mut bytes, ordering);
    encode_query_access_contract(&mut bytes, access_contract);
    encode_query_execution_shape(&mut bytes, execution_shape);
    encode_reduction_discipline(&mut bytes, reduction);
    primitive_terms::encode_usize(&mut bytes, target_count_hint);
    DeterministicQueryPlanKey(first_u128_digest(&bytes))
}

pub(crate) fn deterministic_query_fragment_key_from_canonical_bytes(
    plan_key: DeterministicQueryPlanKey,
    fragment_ordinal: u64,
) -> DeterministicQueryFragmentKey {
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "query.fragment-key.v1");
    encode_u128(&mut bytes, plan_key.0);
    encode_u64(&mut bytes, fragment_ordinal);
    DeterministicQueryFragmentKey(first_u128_digest(&bytes))
}

pub(crate) fn query_result_reduction_digest(
    ordering: QueryOrderingContract,
    entities: &[EntityReadRecord],
    relations: &[RelationReadRecord],
) -> String {
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "query.reduction-result.v1");
    encode_query_ordering_contract(&mut bytes, ordering);
    encode_authoritative_entity_records(&mut bytes, entities);
    encode_authoritative_relation_records(&mut bytes, relations);
    sha256_hex(&bytes)
}

pub(crate) fn query_authoritative_entity_record_digest(record: &EntityReadRecord) -> String {
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "query.entity-record.v1");
    encode_entity_read_record(&mut bytes, record);
    sha256_hex(&bytes)
}

pub(crate) fn query_authoritative_relation_record_digest(record: &RelationReadRecord) -> String {
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "query.relation-record.v1");
    encode_relation_read_record(&mut bytes, record);
    sha256_hex(&bytes)
}

pub(crate) fn query_index_parity_basis_digest(
    access_path: &QueryAccessPath,
    parity_mode: IndexParityMode,
    result: &CanonicalQueryResult,
    plan_key: DeterministicQueryPlanKey,
) -> String {
    let mut bytes = Vec::new();
    encode_string(&mut bytes, "query.index-parity.v1");
    encode_query_access_path(&mut bytes, access_path);
    encode_index_parity_mode(&mut bytes, parity_mode);
    encode_canonical_query_result(&mut bytes, result);
    encode_u128(&mut bytes, plan_key.0);
    sha256_hex(&bytes)
}

fn encode_canonical_query_result(bytes: &mut Vec<u8>, result: &CanonicalQueryResult) {
    encode_query_execution_shape(bytes, result.execution_shape);
    encode_query_ordering_contract(bytes, result.ordering);
    encode_authoritative_entity_records(bytes, &result.entities);
    encode_authoritative_relation_records(bytes, &result.relations);
    encode_string(bytes, &result.reduction_digest);
}

fn first_u128_digest(bytes: &[u8]) -> u128 {
    let digest = Sha256::digest(bytes);
    let mut key_bytes = [0u8; 16];
    key_bytes.copy_from_slice(&digest[..16]);
    u128::from_be_bytes(key_bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
