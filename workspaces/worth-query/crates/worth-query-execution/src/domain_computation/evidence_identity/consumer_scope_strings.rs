use super::scope::WorthQueryEvidenceScope;

pub(crate) fn consumer_kit_evidence_scope_as_str(scope: WorthQueryEvidenceScope) -> &'static str {
    match scope {
        WorthQueryEvidenceScope::ConsumerEvidenceReportField => "consumer-evidence-report-field",
        WorthQueryEvidenceScope::ConsumerEvidenceReport => "consumer-evidence-report",
        WorthQueryEvidenceScope::ConsumerEvidenceReportFieldInventory => {
            "consumer-evidence-report-field-inventory"
        }
        WorthQueryEvidenceScope::ConsumerEvidenceReportDigestParticipation => {
            "consumer-evidence-report-digest-participation"
        }
        WorthQueryEvidenceScope::ConsumerSupportSnapshotSchema => {
            "consumer-support-snapshot-schema"
        }
        WorthQueryEvidenceScope::ConsumerSupportSnapshotRow => "consumer-support-snapshot-row",
        WorthQueryEvidenceScope::ConsumerSupportSnapshotDocument => {
            "consumer-support-snapshot-document"
        }
        WorthQueryEvidenceScope::ConsumerProjectionContractDenial => {
            "consumer-projection-contract-denial"
        }
        WorthQueryEvidenceScope::ConsumerSupportPinContractSchema => {
            "consumer-support-pin-contract-schema"
        }
        WorthQueryEvidenceScope::ConsumerSupportPinVocabulary => "consumer-support-pin-vocabulary",
        WorthQueryEvidenceScope::ConsumerSupportPinRequirement => {
            "consumer-support-pin-requirement"
        }
        WorthQueryEvidenceScope::ConsumerSupportPinObservedRow => {
            "consumer-support-pin-observed-row"
        }
        WorthQueryEvidenceScope::ConsumerSupportPinContract => "consumer-support-pin-contract",
        WorthQueryEvidenceScope::ConsumerSupportPinContractDocument => {
            "consumer-support-pin-contract-document"
        }
        WorthQueryEvidenceScope::ConsumerSupportPinFinding => "consumer-support-pin-finding",
        WorthQueryEvidenceScope::ConsumerSupportPinReport => "consumer-support-pin-report",
        _ => unreachable!("consumer kit scope helper called with non-consumer scope"),
    }
}
