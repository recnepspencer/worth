use crate::diagnostics::history::BridgeHistoricalEvaluationRecordIdentity;
use crate::diagnostics::BridgeDiagnosticsFacade;
use crate::identity::BridgeIdentityEvidence;
use crate::routing::BridgeRouteIdentity;

use super::super::digest_basis::retained_truth_label_part;
use super::super::digest_basis::{
    compose_retained_causal_mapping_evidence_identity, retained_mapping_bridge_identity_part,
    RetainedCausalMappingDigestArtifact,
};

pub(crate) fn route_record_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Result<
    Option<BridgeIdentityEvidence>,
    crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal,
> {
    facade
        .route_record_for_route_identity(&BridgeRouteIdentity::from_reference_evidence(
            reference_identity.revalidate_bridge_retained_reference(),
        ))
        .map(|record| {
            compose_retained_causal_mapping_evidence_identity(
                RetainedCausalMappingDigestArtifact::RouteRecord,
                &[
                    retained_mapping_bridge_identity_part(record.route_identity()),
                    retained_mapping_bridge_identity_part(record.invalidation_identity()),
                    retained_truth_label_part(record.source_commit().canonical_label()),
                ],
            )
        })
        .transpose()
}

pub(crate) fn historical_evaluation_record_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Result<
    Option<BridgeIdentityEvidence>,
    crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal,
> {
    facade
        .historical_record_for_record_identity(
            &BridgeHistoricalEvaluationRecordIdentity::from_reference_evidence(
                reference_identity.revalidate_bridge_retained_reference(),
            ),
        )
        .map(|record| {
            compose_retained_causal_mapping_evidence_identity(
                RetainedCausalMappingDigestArtifact::HistoricalEvaluationRecord,
                &[
                    retained_mapping_bridge_identity_part(record.record_identity()),
                    retained_mapping_bridge_identity_part(
                        record.decision_log().decision_log_identity(),
                    ),
                    retained_truth_label_part(
                        record.decision_log().snapshot_identity().canonical_label(),
                    ),
                ],
            )
        })
        .transpose()
}
