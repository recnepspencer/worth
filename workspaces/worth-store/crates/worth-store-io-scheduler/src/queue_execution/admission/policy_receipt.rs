use worth_foundational::{
    FoundationalPerformanceBoundary, FoundationalPerformanceEvidenceStrength,
    FoundationalPerformanceFallbackDebtPosture, FoundationalPerformanceWorkClass,
    FoundationalPolicyAdmissionReceipt,
};

use super::{
    QueueDurabilityClass, QueueExecutionAdmissionDenial, QueueWorkClass, QueueWorkDeclaration,
};

/// Policy admission sealed to one exact scheduler workload declaration.
#[derive(Debug)]
pub struct QueuePolicyAdmissionReceipt {
    work: QueueWorkDeclaration,
    foundational: FoundationalPolicyAdmissionReceipt,
}

impl QueuePolicyAdmissionReceipt {
    pub(super) const fn work(&self) -> &QueueWorkDeclaration {
        &self.work
    }

    pub(super) fn into_parts(self) -> (QueueWorkDeclaration, FoundationalPolicyAdmissionReceipt) {
        (self.work, self.foundational)
    }

    pub(super) fn foundational(&self) -> &FoundationalPolicyAdmissionReceipt {
        &self.foundational
    }
}

pub fn admit_queue_policy_receipt(
    work: QueueWorkDeclaration,
    receipt: FoundationalPolicyAdmissionReceipt,
) -> Result<QueuePolicyAdmissionReceipt, QueueExecutionAdmissionDenial> {
    let expected_work = expected_work_class(&work);
    let exact_policy_context = receipt.boundary()
        == FoundationalPerformanceBoundary::AuthoritativeExecution
        && receipt.evidence_strength()
            == FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission
        && receipt.claim().fallback_debt() == FoundationalPerformanceFallbackDebtPosture::Verified
        && receipt.stronger_evidence_still_required()
            == FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt
        && receipt.included_work() == [expected_work]
        && !receipt.excluded_work().contains(&expected_work)
        && receipt.denied_work().is_empty()
        && receipt.widened_work().is_empty();
    if !exact_policy_context {
        return Err(QueueExecutionAdmissionDenial::PolicyReceiptContextMismatch { expected_work });
    }
    Ok(QueuePolicyAdmissionReceipt {
        work,
        foundational: receipt,
    })
}

pub(super) const fn expected_work_class(
    work: &QueueWorkDeclaration,
) -> FoundationalPerformanceWorkClass {
    if let QueueWorkClass::Background(class) = work.class() {
        return expected_background_work_class(class);
    }
    match work.durability_class() {
        QueueDurabilityClass::ReadOnly => FoundationalPerformanceWorkClass::AuthoritativeRead,
        QueueDurabilityClass::BufferedWrite | QueueDurabilityClass::WalCommit => {
            FoundationalPerformanceWorkClass::AuthoritativeMutation
        }
        QueueDurabilityClass::PlatformDurable => {
            FoundationalPerformanceWorkClass::PublicationDelivery
        }
    }
}

const fn expected_background_work_class(
    class: crate::BackgroundIoPressureClass,
) -> FoundationalPerformanceWorkClass {
    match class {
        crate::BackgroundIoPressureClass::CheckpointFlush
        | crate::BackgroundIoPressureClass::CompactionRewrite
        | crate::BackgroundIoPressureClass::IngestPressure
        | crate::BackgroundIoPressureClass::BlobReclaimPressure
        | crate::BackgroundIoPressureClass::MigrationPressure => {
            FoundationalPerformanceWorkClass::AuthoritativeMutation
        }
        crate::BackgroundIoPressureClass::ScrubScan
        | crate::BackgroundIoPressureClass::ReplicationPrepRead
        | crate::BackgroundIoPressureClass::BackupPrepRead
        | crate::BackgroundIoPressureClass::RepairScan
        | crate::BackgroundIoPressureClass::VerificationPressure => {
            FoundationalPerformanceWorkClass::ValidationPlanning
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{expected_background_work_class, FoundationalPerformanceWorkClass};
    use crate::BackgroundIoPressureClass;

    #[test]
    fn checkpoint_is_authoritative_mutation_not_validation_planning() {
        assert_eq!(
            expected_background_work_class(BackgroundIoPressureClass::CheckpointFlush),
            FoundationalPerformanceWorkClass::AuthoritativeMutation
        );
        assert_eq!(
            expected_background_work_class(BackgroundIoPressureClass::CompactionRewrite),
            FoundationalPerformanceWorkClass::AuthoritativeMutation
        );
        assert_eq!(
            expected_background_work_class(BackgroundIoPressureClass::VerificationPressure),
            FoundationalPerformanceWorkClass::ValidationPlanning
        );
        assert_eq!(
            expected_background_work_class(BackgroundIoPressureClass::IngestPressure),
            FoundationalPerformanceWorkClass::AuthoritativeMutation
        );
        assert_eq!(
            expected_background_work_class(BackgroundIoPressureClass::MigrationPressure),
            FoundationalPerformanceWorkClass::AuthoritativeMutation
        );
        assert_eq!(
            expected_background_work_class(BackgroundIoPressureClass::BlobReclaimPressure),
            FoundationalPerformanceWorkClass::AuthoritativeMutation
        );
    }
}
