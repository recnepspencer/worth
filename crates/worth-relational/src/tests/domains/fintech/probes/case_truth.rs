use std::collections::BTreeMap;

use crate::facade::identity::VersionId;
use crate::facade::runtime::EntityReadRecord;
use crate::facade::snapshots::SnapshotHandle;
use crate::tests::support::{field_key, read_entity_field};
use worth_foundational::facade::{
    AspectContractRevision, AspectKey, AuthoritativeRecordAspectState,
    ContractValidatedAspectValueView, FieldKey,
};

use super::super::fixture::{FintechCaseRole, FintechWorld};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProbeStage {
    Baseline,
    PostMutation,
    PostReplay,
    PostRecovery,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CaseTruthProbe {
    pub(crate) case_role: FintechCaseRole,
    pub(crate) stage: ProbeStage,
    pub(crate) snapshot_id: u64,
    pub(crate) entity_count: usize,
    pub(crate) relation_count: usize,
    pub(crate) corrected_trade_count: usize,
    pub(crate) repaired_settlement_count: usize,
    pub(crate) open_breach_count: usize,
    pub(crate) audit_record_count: usize,
    pub(crate) aspect_state_fingerprints: BTreeMap<String, Vec<AspectStateFingerprint>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AspectStateFingerprint {
    aspect_key: AspectKey,
    contract_revision: AspectContractRevision,
    value: AspectStateValueFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AspectStateValueFingerprint {
    Scalar(Vec<u8>),
    Struct(Vec<(FieldKey, Vec<u8>)>),
}

pub(crate) fn capture_case_truth_probe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &FintechWorld,
    case: FintechCaseRole,
    stage: ProbeStage,
) -> CaseTruthProbe {
    let snapshot = world.latest_snapshot();
    read_snapshot_probe(execution, world, case, &snapshot, stage)
}

pub(crate) fn read_snapshot_probe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &FintechWorld,
    case: FintechCaseRole,
    snapshot: &SnapshotHandle,
    stage: ProbeStage,
) -> CaseTruthProbe {
    let packet = world.packet_for_case_probe(case, snapshot);
    let result = world.read_query(execution, snapshot, packet);
    CaseTruthProbe {
        case_role: case,
        stage,
        snapshot_id: snapshot.snapshot_id.0,
        entity_count: result.entities.len(),
        relation_count: result.relations.len(),
        corrected_trade_count: result
            .entities
            .iter()
            .filter(|entity| field_is(entity, field_key("corrected"), "true"))
            .count(),
        repaired_settlement_count: result
            .entities
            .iter()
            .filter(|entity| {
                field_is(entity, field_key("entity_type"), "settlement")
                    && field_is(entity, field_key("status"), "repaired")
            })
            .count(),
        open_breach_count: result
            .entities
            .iter()
            .filter(|entity| {
                field_is(entity, field_key("entity_type"), "limit_breach")
                    && field_is(entity, field_key("status"), "open")
            })
            .count(),
        audit_record_count: result
            .entities
            .iter()
            .filter(|entity| field_is(entity, field_key("entity_type"), "audit_record"))
            .count(),
        aspect_state_fingerprints: result
            .entities
            .iter()
            .enumerate()
            .map(|(idx, entity)| (format!("entity-{idx}"), aspect_state_fingerprint(entity)))
            .collect(),
    }
}

pub(crate) fn read_version_probe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &FintechWorld,
    case: FintechCaseRole,
    version_id: VersionId,
    stage: ProbeStage,
) -> CaseTruthProbe {
    let packet = world.packet_for_case_probe(case, &world.latest_snapshot());
    let read = world
        .runtime
        .read_truth()
        .read_version(execution, version_id)
        .expect("read request remains live");
    let fragment = read
        .execute_planned_packet_fragment(
            crate::facade::query::DeterministicQueryPlanKey(0),
            crate::facade::query::QueryOrderingContract::CanonicalRecordRefOrder,
            packet
                .explicit_target_refs()
                .expect("case version probe uses explicit target packet"),
            0,
        )
        .expect("version probe packet fragment");
    let reduced = crate::query::data::reduce_query_fragments(
        crate::facade::query::QueryExecutionShape::BulkPacketized,
        crate::facade::query::QueryOrderingContract::CanonicalRecordRefOrder,
        vec![fragment],
    );
    let entities = reduced.entities;
    let relations = reduced.relations;
    CaseTruthProbe {
        case_role: case,
        stage,
        snapshot_id: 0,
        entity_count: entities.len(),
        relation_count: relations.len(),
        corrected_trade_count: entities
            .iter()
            .filter(|entity| field_is(entity, field_key("corrected"), "true"))
            .count(),
        repaired_settlement_count: entities
            .iter()
            .filter(|entity| {
                field_is(entity, field_key("entity_type"), "settlement")
                    && field_is(entity, field_key("status"), "repaired")
            })
            .count(),
        open_breach_count: entities
            .iter()
            .filter(|entity| {
                field_is(entity, field_key("entity_type"), "limit_breach")
                    && field_is(entity, field_key("status"), "open")
            })
            .count(),
        audit_record_count: entities
            .iter()
            .filter(|entity| field_is(entity, field_key("entity_type"), "audit_record"))
            .count(),
        aspect_state_fingerprints: entities
            .iter()
            .enumerate()
            .map(|(idx, entity)| (format!("entity-{idx}"), aspect_state_fingerprint(entity)))
            .collect(),
    }
}

fn aspect_state_fingerprint(entity: &EntityReadRecord) -> Vec<AspectStateFingerprint> {
    entity
        .authoritative_aspect_state
        .as_ref()
        .map(fingerprint_authoritative_aspect_state)
        .unwrap_or_default()
}

fn fingerprint_authoritative_aspect_state(
    state: &AuthoritativeRecordAspectState,
) -> Vec<AspectStateFingerprint> {
    state
        .aspects()
        .entries()
        .map(|(aspect_key, validated_value)| AspectStateFingerprint {
            aspect_key: aspect_key.clone(),
            contract_revision: validated_value.contract_revision(),
            value: match validated_value.view() {
                ContractValidatedAspectValueView::Scalar(value) => {
                    AspectStateValueFingerprint::Scalar(encode_aspect_value(value))
                }
                ContractValidatedAspectValueView::Struct(value) => {
                    AspectStateValueFingerprint::Struct(
                        value
                            .fields()
                            .map(|(field_key, field_value)| {
                                (field_key.clone(), encode_aspect_value(field_value))
                            })
                            .collect(),
                    )
                }
            },
        })
        .collect()
}

fn encode_aspect_value(value: &worth_foundational::facade::AspectValue) -> Vec<u8> {
    crate::aspect_wire::encode_aspect_value(value)
}

fn field_is(entity: &EntityReadRecord, field: FieldKey, expected: &str) -> bool {
    read_entity_field(entity, field) == Some(expected.to_string())
}
