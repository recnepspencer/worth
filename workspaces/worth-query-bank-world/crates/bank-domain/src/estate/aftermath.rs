//! Estate aftermath declarations through the generic Query aftermath contract.
//!
//! Published law-14 posture names are derived at installation. This module does
//! not declare `Reversible`, `Compensatable`, `Reconcilable`, or `Irreversible`
//! directly, and reads carry no aftermath contract.

use worth_query_decl::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract, DeclaredCompensation,
    DeclaredCorrectionMechanism, DeclaredReconciliationProcedure,
};

use crate::schema::BankSchema;

use super::EstateCapabilityOperation;

/// The one correlation family estate death notices escape through.
///
/// The application operation definition's external-effect slot is the sole
/// declaration of it. The aftermath contract deliberately cannot name a rail,
/// so an aftermath that disagreed with the lane the outbox correlates to is
/// unrepresentable rather than merely discouraged (Q8.25-C1).
pub const ESTATE_DEATH_NOTICE_RAIL: &str = "estate-death-notice-rail";

/// Declared aftermath for one estate capability operation.
///
/// `None` means the operation is not a mutation and carries no aftermath.
pub fn declared_aftermath_for(
    operation: EstateCapabilityOperation,
) -> Option<DeclaredApplicationAftermathContract<BankSchema>> {
    match operation {
        EstateCapabilityOperation::ViewRestrictedEstate => None,
        EstateCapabilityOperation::NotifyDeath
        | EstateCapabilityOperation::RetransmitDeathNotice => Some(
            DeclaredApplicationAftermathContract::runtime_with_external_owner(
                DeclaredCorrectionMechanism::Compensation(
                    DeclaredCompensation::new(
                        "confirm-death-notice-with-authority",
                        DeclaredAftermathPostcondition::BusinessPostcondition {
                            identity: "death-notice-confirmed".into(),
                        },
                    ),
                ),
                DeclaredReconciliationProcedure::new("confirm-death-notice-with-authority"),
            ),
        ),
        EstateCapabilityOperation::DisburseEstate => {
            Some(DeclaredApplicationAftermathContract::runtime_alone(
                DeclaredCorrectionMechanism::Compensation(
                    DeclaredCompensation::new(
                        "compensating-estate-journal",
                        DeclaredAftermathPostcondition::BusinessPostcondition {
                            identity: "estate-journal-settled".into(),
                        },
                    ),
                ),
            ))
        }
        EstateCapabilityOperation::FreezeAccount
        | EstateCapabilityOperation::DelegateCapability
        | EstateCapabilityOperation::RequestEmergencyAccess
        | EstateCapabilityOperation::RevokeCapability
        | EstateCapabilityOperation::ApproveEmergencyAccess
        | EstateCapabilityOperation::OpenEstateCase
        | EstateCapabilityOperation::RecognizeExecutor
        | EstateCapabilityOperation::RevokeEmergencyAccess
        | EstateCapabilityOperation::CompleteMandatoryReview
        | EstateCapabilityOperation::ReleaseEstate => {
            Some(DeclaredApplicationAftermathContract::not_correctable())
        }
    }
}
