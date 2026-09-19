use worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryRecoveryDurabilityPosture,
    WorthQueryRecoveryInspectionView,
};

use super::external_effect::publish_external_effect;
use super::outcome::publish_posture;
use super::{
    WorthQueryPublishedApplicationAftermath, WorthQueryPublishedCanonicalWork,
    WorthQueryPublishedRecoveryDurability, WorthQueryPublishedRecoverySupport,
    WorthQueryPublishedRecoverySupportTruth,
};

/// Publishes only a sealed execution commit.
///
/// ```compile_fail
/// use worth_query_installation::facade::PublishedAftermathPosture;
/// use worth_query_publication::facade::application_aftermath::publish_application_aftermath;
///
/// let copied = PublishedAftermathPosture::Reconcilable;
/// let _ = publish_application_aftermath(&copied);
/// ```
pub fn publish_application_aftermath(
    receipt: &WorthQueryApplicationCommitReceipt,
) -> WorthQueryPublishedApplicationAftermath {
    WorthQueryPublishedApplicationAftermath::new(
        receipt.published_aftermath_posture().map(publish_posture),
        publish_external_effect(receipt),
    )
}

/// Publishes only a disclosure-admitted runtime inspection view.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryOpaqueRecoveryWireIdentity;
/// use worth_query_publication::facade::application_aftermath::publish_recovery_support;
///
/// let wire: WorthQueryOpaqueRecoveryWireIdentity = todo!();
/// let _ = publish_recovery_support(&wire);
/// ```
pub const fn publish_recovery_support(
    inspection: &WorthQueryRecoveryInspectionView,
) -> WorthQueryPublishedRecoverySupport {
    WorthQueryPublishedRecoverySupport::new(
        publish_support_truth(inspection.support_truth()),
        publish_posture(inspection.published_posture()),
        publish_durability(inspection.durability()),
        WorthQueryPublishedCanonicalWork::from_owner(inspection.recovery_inspection_work()),
    )
}

const fn publish_support_truth(
    truth: FoundationalBoundaryEvidenceSupportTruthKind,
) -> WorthQueryPublishedRecoverySupportTruth {
    match truth {
        FoundationalBoundaryEvidenceSupportTruthKind::EvidenceBundle => {
            WorthQueryPublishedRecoverySupportTruth::EvidenceBundle
        }
        FoundationalBoundaryEvidenceSupportTruthKind::CertificationSummary => {
            WorthQueryPublishedRecoverySupportTruth::CertificationSummary
        }
        FoundationalBoundaryEvidenceSupportTruthKind::ParityArtifact => {
            WorthQueryPublishedRecoverySupportTruth::ParityArtifact
        }
        FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport => {
            WorthQueryPublishedRecoverySupportTruth::DegradedRecoveryReport
        }
        FoundationalBoundaryEvidenceSupportTruthKind::StaleBasisDisclosure => {
            WorthQueryPublishedRecoverySupportTruth::StaleBasisDisclosure
        }
        FoundationalBoundaryEvidenceSupportTruthKind::TransientLifecycleEvidence => {
            WorthQueryPublishedRecoverySupportTruth::TransientLifecycleEvidence
        }
        FoundationalBoundaryEvidenceSupportTruthKind::ResidualDebtStatement => {
            WorthQueryPublishedRecoverySupportTruth::ResidualDebtStatement
        }
    }
}

const fn publish_durability(
    durability: WorthQueryRecoveryDurabilityPosture,
) -> WorthQueryPublishedRecoveryDurability {
    match durability {
        WorthQueryRecoveryDurabilityPosture::StoreCapabilityRequired => {
            WorthQueryPublishedRecoveryDurability::StoreCapabilityRequired
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_disclosure_admitted_support_truth_maps_exactly() {
        use FoundationalBoundaryEvidenceSupportTruthKind as Owner;
        use WorthQueryPublishedRecoverySupportTruth as Published;

        for (owner, published) in [
            (Owner::EvidenceBundle, Published::EvidenceBundle),
            (Owner::CertificationSummary, Published::CertificationSummary),
            (Owner::ParityArtifact, Published::ParityArtifact),
            (
                Owner::DegradedRecoveryReport,
                Published::DegradedRecoveryReport,
            ),
            (Owner::StaleBasisDisclosure, Published::StaleBasisDisclosure),
            (
                Owner::TransientLifecycleEvidence,
                Published::TransientLifecycleEvidence,
            ),
            (
                Owner::ResidualDebtStatement,
                Published::ResidualDebtStatement,
            ),
        ] {
            assert_eq!(publish_support_truth(owner), published);
        }
    }

    #[test]
    fn disclosure_admitted_durability_maps_exactly() {
        assert_eq!(
            publish_durability(WorthQueryRecoveryDurabilityPosture::StoreCapabilityRequired),
            WorthQueryPublishedRecoveryDurability::StoreCapabilityRequired
        );
    }
}
