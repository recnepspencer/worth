use crate::facade::{
    BridgeCausalEvidenceBinding, BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner,
    BridgeCausalEvidenceReference, BridgeCausalEvidenceReferenceIdentity,
    BridgeMappedWritebackFamilyInput, BridgeRouteResultSummary, BridgeWritebackExecutionRecord,
    BridgeWritebackFamilyAdmissionRecord, BridgeWritebackMapperEnvelope,
    BridgeWritebackMapperRecord, BridgeWritebackReplayRecord,
};

pub(super) fn bridge_reference(
    identity: BridgeCausalEvidenceReferenceIdentity,
) -> BridgeCausalEvidenceReference {
    let family = identity.family();
    BridgeCausalEvidenceReference::new(BridgeCausalEvidenceOwner::RuntimeBridge, family, identity)
        .expect("bridge reference should be valid")
}

pub(super) fn query_observation_reference(
    identity: BridgeCausalEvidenceReferenceIdentity,
) -> BridgeCausalEvidenceReference {
    BridgeCausalEvidenceReference::new(
        BridgeCausalEvidenceOwner::Query,
        BridgeCausalEvidenceFamily::QueryObservation,
        identity,
    )
    .expect("query observation reference should be valid")
}

pub(super) fn missing_bridge_reference(
    family: BridgeCausalEvidenceFamily,
    identity: &str,
) -> BridgeCausalEvidenceReference {
    bridge_reference(
        BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
            family,
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(identity),
        )
        .expect("bridge reference identity should be valid"),
    )
}

pub(super) fn bridge_route_reference(
    route_summary: &BridgeRouteResultSummary,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeRoute,
        route_summary.route_identity().as_str(),
    )
}

pub(super) fn bridge_writeback_admission_reference(
    record: &BridgeWritebackFamilyAdmissionRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeWritebackAdmission,
        record.record_identity().as_str(),
    )
}

pub(super) fn bridge_writeback_mapper_envelope_reference(
    envelope: &BridgeWritebackMapperEnvelope,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeWritebackMapperEnvelope,
        envelope.envelope_identity().as_str(),
    )
}

pub(super) fn bridge_writeback_mapped_input_reference(
    mapped_input: &BridgeMappedWritebackFamilyInput,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeWritebackMappedFamilyInput,
        mapped_input.mapped_input_identity().as_str(),
    )
}

pub(super) fn bridge_writeback_mapper_record_reference(
    record: &BridgeWritebackMapperRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeWritebackMapper,
        record.record_identity().as_str(),
    )
}

pub(super) fn bridge_writeback_execution_reference(
    record: &BridgeWritebackExecutionRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeWritebackExecution,
        record.record_identity().as_str(),
    )
}

pub(super) fn bridge_writeback_replay_reference(
    record: &BridgeWritebackReplayRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeWritebackReplay,
        record.record_identity().as_str(),
    )
}

pub(super) fn binding_for<'a>(
    bindings: &'a [BridgeCausalEvidenceBinding],
    family: BridgeCausalEvidenceFamily,
    reference_identity: &str,
) -> &'a BridgeCausalEvidenceBinding {
    bindings
        .iter()
        .find(|binding| {
            binding.owner() == BridgeCausalEvidenceOwner::RuntimeBridge
                && binding.family() == family
                && binding.reference_evidence_identity().as_str() == reference_identity
        })
        .expect("expected writeback causal binding should be present")
}

pub(super) fn writeback_admission_digest(
    record: &crate::facade::BridgeWritebackFamilyAdmissionRecord,
) -> String {
    crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::writeback::writeback_admission_digest(record)
        .expect("bounded digest fixture").as_str()
        .to_string()
}

pub(super) fn writeback_mapper_envelope_digest(
    envelope: &crate::facade::BridgeWritebackMapperEnvelope,
) -> String {
    crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::writeback::writeback_mapper_envelope_artifact_digest(envelope)
        .expect("bounded digest fixture").as_str()
        .to_string()
}

pub(super) fn writeback_mapped_input_digest(
    mapped_input: &crate::facade::BridgeMappedWritebackFamilyInput,
) -> String {
    crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::writeback::writeback_mapped_family_input_artifact_digest(mapped_input)
        .expect("bounded digest fixture").as_str()
        .to_string()
}

pub(super) fn writeback_mapper_record_digest(
    record: &crate::facade::BridgeWritebackMapperRecord,
) -> String {
    crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::writeback::writeback_mapper_record_artifact_digest(record)
        .expect("bounded digest fixture").as_str()
        .to_string()
}

pub(super) fn writeback_execution_digest(
    record: &crate::facade::BridgeWritebackExecutionRecord,
) -> String {
    crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::writeback::writeback_execution_artifact_digest(record)
        .expect("bounded digest fixture").as_str()
        .to_string()
}

pub(super) fn writeback_replay_digest(
    record: &crate::facade::BridgeWritebackReplayRecord,
) -> String {
    crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::writeback::writeback_replay_artifact_digest(record)
        .expect("bounded digest fixture").as_str()
        .to_string()
}
