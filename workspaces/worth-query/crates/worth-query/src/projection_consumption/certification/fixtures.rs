use crate::authorized_projection::{
    AuthorizedProjectionArtifact, AuthorizedProjectionCounters, MaskedProjectionArtifact,
    PolicyFieldInfluenceSet,
};
use crate::projection_consumption::identity::{
    compose_certified_source_digest, compose_certified_source_receipt_digest,
};
use crate::projection_consumption::{
    declare_projection_consumption, evaluate_projection_consumption_eligibility,
    ProjectMaterializedFacts, ProjectionConsumptionAuthoringSurface,
    ProjectionConsumptionBindingContext, ProjectionConsumptionDeclaration,
    ProjectionConsumptionEligibility, ProjectionConsumptionSource,
};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, ScalarAspectType};
use worth_runtime_bridge::facade::{
    encode_snapshot_aspect_read_value, materialize_relational_authoritative_row_set,
    project_relational_grouped_truth, RelationalAuthoritativeRowSetArtifact,
    RelationalGroupedProjectionArtifact,
};
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
    SnapshotReadContract, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
    SnapshotReadRequest, TruthSnapshotIdentity,
};

use super::super::consumed::ConsumedProjectionFactSet;
use super::super::contracts::MaterializedProjectionContract;
use super::super::envelope::SelfDescribingProjectionConsumptionEnvelope;
use super::super::receipt::ProjectionConsumptionReceipt;
use super::grouped_projection_contract::grouped_projection_contract;

const QUERY_DIGEST: &str = "query:projection_consumption_certification";
const RESULT_SHAPE_DIGEST: &str = "result-shape:projection_consumption_certification";
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectionConsumptionCertifiedLifecycle {
    declaration: ProjectionConsumptionDeclaration,
    contract: MaterializedProjectionContract,
    facts: ConsumedProjectionFactSet,
    receipt: ProjectionConsumptionReceipt,
    envelope: SelfDescribingProjectionConsumptionEnvelope,
}

impl ProjectionConsumptionCertifiedLifecycle {
    pub fn declaration(&self) -> &ProjectionConsumptionDeclaration {
        &self.declaration
    }

    pub fn contract(&self) -> &MaterializedProjectionContract {
        &self.contract
    }

    pub fn facts(&self) -> &ConsumedProjectionFactSet {
        &self.facts
    }

    pub fn receipt(&self) -> &ProjectionConsumptionReceipt {
        &self.receipt
    }

    pub fn envelope(&self) -> &SelfDescribingProjectionConsumptionEnvelope {
        &self.envelope
    }

    pub(super) fn into_completed(self) -> super::super::CompletedProjectionFactConsumption {
        super::super::CompletedProjectionFactConsumption::certification(
            self.declaration,
            self.contract,
            self.facts,
            self.receipt,
        )
    }
}

pub fn control_row_set_lifecycle(row_count: usize) -> ProjectionConsumptionCertifiedLifecycle {
    let row_set = certification_row_set(row_count);
    let source = ProjectionConsumptionSource::from_relational_row_set(&row_set);
    let declaration = declare_projection_consumption(
        source,
        control_binding(&[
            "identity.id",
            "profile.display_name",
            "status.lane",
            "metrics.priority",
        ]),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
    )
    .expect("control declaration should author cleanly");
    let contract = admitted_contract(&declaration);
    let facts = contract
        .extract_from_relational_row_set(&row_set)
        .expect("control extraction should succeed");
    let receipt = facts.issue_receipt();
    let envelope = receipt.projection_consumption_envelope();
    ProjectionConsumptionCertifiedLifecycle {
        declaration,
        contract,
        facts,
        receipt,
        envelope,
    }
}

pub fn grouped_worth_lifecycle(row_count: usize) -> ProjectionConsumptionCertifiedLifecycle {
    let grouped = certification_grouped_projection(row_count);
    let source = ProjectionConsumptionSource::from_relational_grouped_projection(&grouped);
    let declaration = declare_projection_consumption(
        source,
        control_binding(&["identity.id", "status.lane"]),
        ProjectMaterializedFacts::declare()
            .memberships()
            .relation_endpoints()
            .view_local_identities(),
    )
    .expect("grouped declaration should author cleanly");
    let contract = admitted_contract(&declaration);
    let facts = contract
        .extract_from_relational_grouped_projection(&grouped)
        .expect("grouped extraction should succeed");
    let receipt = facts.issue_receipt();
    let envelope = receipt.projection_consumption_envelope();
    ProjectionConsumptionCertifiedLifecycle {
        declaration,
        contract,
        facts,
        receipt,
        envelope,
    }
}

pub fn parity_row_set_lifecycle(row_count: usize) -> ProjectionConsumptionCertifiedLifecycle {
    let row_set = certification_row_set(row_count);
    let declaration = ProjectMaterializedFacts::declare()
        .entity_identities()
        .display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
            ]),
        )
        .source(
            ProjectionConsumptionAuthoringSurface::from_relational_row_set(
                &row_set,
                RESULT_SHAPE_DIGEST,
                &control_authorized_projection(&[
                    "identity.id",
                    "profile.display_name",
                    "status.lane",
                    "metrics.priority",
                ]),
            ),
        )
        .build()
        .expect("parity declaration should author cleanly");
    let contract = admitted_contract(&declaration);
    let facts = contract
        .extract_from_relational_row_set(&row_set)
        .expect("parity extraction should succeed");
    let receipt = facts.issue_receipt();
    let envelope = receipt.projection_consumption_envelope();
    ProjectionConsumptionCertifiedLifecycle {
        declaration,
        contract,
        facts,
        receipt,
        envelope,
    }
}

pub fn denied_masked_field_failure_digest() -> String {
    let row_set = certification_row_set(2);
    let source = ProjectionConsumptionSource::from_relational_row_set(&row_set);
    let declaration = declare_projection_consumption(
        source,
        control_binding(&["identity.id"]),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
    )
    .expect("denial declaration should author cleanly");
    match evaluate_projection_consumption_eligibility(&declaration) {
        ProjectionConsumptionEligibility::Denied(denied) => denied.failure_digest().to_string(),
        other => panic!("expected denied certification failure, got {other:?}"),
    }
}

pub fn source_mismatch_failure_digest() -> String {
    let row_set = certification_row_set(2);
    let source = ProjectionConsumptionSource::from_relational_row_set(&row_set);
    let declaration = declare_projection_consumption(
        source,
        control_binding(&["identity.id", "status.lane"]),
        ProjectMaterializedFacts::declare().source_references(),
    )
    .expect("source mismatch declaration should author cleanly");
    match evaluate_projection_consumption_eligibility(&declaration) {
        ProjectionConsumptionEligibility::SourceMismatch(mismatch) => {
            mismatch.failure_digest().to_string()
        }
        other => panic!("expected source mismatch certification failure, got {other:?}"),
    }
}

pub fn source_digest(contract: &MaterializedProjectionContract) -> String {
    compose_certified_source_digest(contract)
}

pub fn source_receipt_digest(contract: &MaterializedProjectionContract) -> String {
    compose_certified_source_receipt_digest(contract, &compose_certified_source_digest(contract))
}

fn admitted_contract(
    declaration: &ProjectionConsumptionDeclaration,
) -> MaterializedProjectionContract {
    match evaluate_projection_consumption_eligibility(declaration) {
        ProjectionConsumptionEligibility::Admitted(admitted) => admitted.bind_contract(),
        ProjectionConsumptionEligibility::AdmittedWithWarnings(admitted, _) => {
            admitted.bind_contract()
        }
        other => panic!("expected admitted certification lane, got {other:?}"),
    }
}

fn control_binding(visible_fields: &[&str]) -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::from_result_shape_digest(
        RESULT_SHAPE_DIGEST,
        &control_authorized_projection(visible_fields),
    )
}

fn control_authorized_projection(visible_fields: &[&str]) -> AuthorizedProjectionArtifact {
    AuthorizedProjectionArtifact::new(
        QUERY_DIGEST,
        RESULT_SHAPE_DIGEST,
        "policy:projection-consumption-certification",
        "tenant-schema:projection-consumption-certification",
        visible_fields.iter().map(field_path).collect(),
        MaskedProjectionArtifact::new(
            vec![field_path(&"masked.field")],
            vec![field_path(&"masked.family")],
        ),
        "narrowed-result-shape:projection-consumption-certification".to_string(),
        PolicyFieldInfluenceSet::new(&["policy-influence:test".to_string()], 1),
        AuthorizedProjectionCounters::default(),
    )
}

pub fn certification_grouped_projection(row_count: usize) -> RelationalGroupedProjectionArtifact {
    project_relational_grouped_truth(
        &certification_row_set(row_count),
        grouped_projection_contract("status", "identity.id", "status.lane"),
    )
    .expect("grouped projection certification fixture")
}

pub fn certification_row_set(row_count: usize) -> RelationalAuthoritativeRowSetArtifact {
    let mut reads = Vec::new();
    let mut records = Vec::new();
    for index in 0..row_count {
        let entity = RelationalBridgeRecordIdentityParts::entity(1, (index + 1) as u64, 1);
        let task = format!("task-{}", index + 1);
        let lane = if index % 2 == 0 { "todo" } else { "doing" };
        let name = format!("Task {}", index + 1);
        let identity_read = string_read(entity, "identity.id");
        let lane_read = string_read(entity, "status.lane");
        let display_name_read = string_read(entity, "profile.display_name");
        let priority_read = scalar_read(entity, "metrics.priority", ScalarAspectType::UInt64);
        records.push(SnapshotReadRecord::for_request(
            &identity_read,
            aspect_value(
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(task),
            ),
        ));
        records.push(SnapshotReadRecord::for_request(
            &lane_read,
            aspect_value(
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(lane),
            ),
        ));
        records.push(SnapshotReadRecord::for_request(
            &display_name_read,
            aspect_value(
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(name),
            ),
        ));
        records.push(SnapshotReadRecord::for_request(
            &priority_read,
            aspect_value(AspectValue::UInt64((index + 1) as u64)),
        ));
        reads.extend([identity_read, lane_read, display_name_read, priority_read]);
    }
    materialize_relational_authoritative_row_set(
        &SnapshotReadPacket::new(reads),
        &SnapshotReadPacketResult::new(
            TruthSnapshotIdentity::from_relational_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(row_count as u64, 1),
            ),
            records,
        ),
    )
    .expect("row set certification fixture")
}

fn snapshot_aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid certification snapshot aspect key")
}

fn string_read(entity: RelationalBridgeRecordIdentityParts, aspect: &str) -> SnapshotReadRequest {
    scalar_read(entity, aspect, ScalarAspectType::String)
}

fn scalar_read(
    entity: RelationalBridgeRecordIdentityParts,
    aspect: &str,
    scalar_type: ScalarAspectType,
) -> SnapshotReadRequest {
    SnapshotReadRequest::for_relational_record(
        entity,
        SnapshotReadContract::scalar(snapshot_aspect_key(aspect), scalar_type),
    )
}

fn aspect_value(value: AspectValue) -> AspectValue {
    encode_snapshot_aspect_read_value(&value)
}

fn field_path(field: &&str) -> crate::authorized_projection::AuthorizedProjectionFieldPath {
    let (aspect, field) = field
        .split_once('.')
        .expect("certification field path must include aspect and field");
    crate::authorized_projection::AuthorizedProjectionFieldPath::from_native_keys(
        AspectKey::new(aspect).expect("certification aspect key should be foundational"),
        FieldKey::new(field).expect("certification field key should be foundational"),
    )
}
