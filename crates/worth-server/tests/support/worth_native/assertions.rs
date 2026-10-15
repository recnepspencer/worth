#![allow(dead_code)]

use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServer, WorthServerAdmittedDirectDeclaration, WorthServerOperatorEvidenceRecord,
    WorthServerQuerySupportPosture, WorthServerResponseEnvelope, WorthServerWorthNativeSession,
};

use crate::shared_support_worth_native_runtime::worth_native_session_input_builder;

pub(crate) fn worth_native_session(server: &WorthServer) -> WorthServerWorthNativeSession {
    match server.worth_native().session(
        worth_native_session_input_builder()
            .build()
            .expect("session input should validate"),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected successful Worth-native session, got {other:?}"),
    }
}

pub(crate) fn admitted_named_read(
    session: &WorthServerWorthNativeSession,
    operation_name: &str,
) -> WorthServerAdmittedDirectDeclaration {
    session
        .declarations()
        .read(worth_server::WorthServerDirectDeclaration::named_read(
            operation_name,
        ))
        .expect("direct declaration should prepare")
        .admit()
        .expect("direct declaration should admit")
}

pub(crate) fn family_contract_digest(posture: &WorthServerQuerySupportPosture) -> &str {
    match posture {
        WorthServerQuerySupportPosture::ProductIndependent { label } => label,
        WorthServerQuerySupportPosture::PrimaryGraphApplicationSupported { basis_token } => {
            basis_token
        }
        WorthServerQuerySupportPosture::QueryReadSupported { family_contract }
        | WorthServerQuerySupportPosture::DirectReadSupported { family_contract }
        | WorthServerQuerySupportPosture::DirectStateSupported { family_contract }
        | WorthServerQuerySupportPosture::DirectInspectionSupported { family_contract }
        | WorthServerQuerySupportPosture::DirectProjectionSupported { family_contract }
        | WorthServerQuerySupportPosture::DirectMutationSupported { family_contract }
        | WorthServerQuerySupportPosture::QueryMutationSupported { family_contract } => {
            family_contract.contract_digest()
        }
        WorthServerQuerySupportPosture::DownstreamDeliverySupported {
            family_contract, ..
        }
        | WorthServerQuerySupportPosture::RuntimeBackedResumeSupported {
            family_contract, ..
        }
        | WorthServerQuerySupportPosture::DurableResumeSupported {
            family_contract, ..
        } => family_contract.contract_digest(),
    }
}

pub(crate) fn response_provenance_digest(response: &WorthServerResponseEnvelope) -> String {
    let provenance = response.provenance();
    format!(
        "locality={:?};freshness={:?};source={:?};authority={};strategy={};profile={};comparison={};canonical={};support_contexts={}",
        provenance.locality(),
        provenance.freshness_posture(),
        provenance.source_basis().kind(),
        provenance.authority_path().is_some(),
        provenance.strategy_basis().is_some(),
        provenance.profile_basis().is_some(),
        provenance.comparison_basis().is_some(),
        provenance.canonical_digest_basis().is_some(),
        provenance.support_context_attachments().len()
    )
}

pub(crate) fn direct_provenance_digest(
    provenance: &worth_server::WorthServerDirectProvenance,
) -> String {
    format!(
        "locality={:?};freshness={:?};source={:?};authority={};strategy={};profile={};comparison={};canonical={};support_contexts={}",
        provenance.locality(),
        provenance.freshness_posture(),
        provenance.source_basis_kind(),
        provenance.has_authority_path(),
        provenance.has_strategy_basis(),
        provenance.has_profile_basis(),
        provenance.has_comparison_basis(),
        provenance.has_canonical_digest_basis(),
        provenance.support_context_attachment_count()
    )
}

pub(crate) fn operator_evidence_record(
    server: &WorthServer,
    response: WorthServerResponseEnvelope,
) -> WorthServerOperatorEvidenceRecord {
    server
        .operator_evidence()
        .record(
            worth_server::WorthServerEvidenceInput::response_envelope(response),
            worth_server::WorthServerEvidenceTransform::operator_default(),
        )
        .expect("operator evidence record should materialize")
}
