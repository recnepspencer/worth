//! Bank command descriptors retain native command authority and preparation separately.
use crate::{BankApplication, BankEstateProgressionDenial};
use bank_domain::{estate::EstateAction, proposals::BankIdempotencyKey, schema::*};
use worth_query_host::facade::{
    application_contribution::WorthQueryAdvancementPhase,
    application_entry::{
        WorthQueryApplicationCapabilityDelegationDenial,
        WorthQueryApplicationCapabilityRevocationDenial, WorthQueryCapabilityCommand,
    },
    primary_graph::{
        WorthQueryAdmittedApplicationOperation, WorthQueryCapabilityRevocationProgram,
        WorthQueryDelegationActivationProgram,
    },
};

/// One owned delegation intent. Advance with fresh Bank requests, then close native custody.
pub type BankDelegationCommand<'runtime> = WorthQueryCapabilityCommand<
    'runtime,
    BankSchema,
    BankApplication,
    WorthQueryApplicationCapabilityDelegationDenial<BankEstateProgressionDenial>,
>;
/// One owned revocation intent. Its performed poststate does not block native keyed recovery.
pub type BankRevocationCommand<'runtime> = WorthQueryCapabilityCommand<
    'runtime,
    BankSchema,
    BankApplication,
    WorthQueryApplicationCapabilityRevocationDenial<BankEstateProgressionDenial>,
>;
/// Repeatable Bank projection preparation; native publication is exclusively the command owner's work.
pub type BankDelegationPreparation<'runtime> = Box<
    dyn FnMut(
            &WorthQueryAdvancementPhase<'_>,
            WorthQueryAdmittedApplicationOperation<
                BankSchema,
                DelegateEstateCapabilityOperation,
                EstateAction,
                EstateCase,
            >,
        ) -> Result<
            WorthQueryDelegationActivationProgram<
                BankSchema,
                DelegateEstateCapabilityOperation,
                EstateAction,
                EstateCase,
            >,
            BankEstateProgressionDenial,
        > + 'runtime,
>;
/// Repeatable exact-target revocation projection, without a retained request or capability grant.
pub type BankRevocationPreparation<'runtime> = Box<
    dyn FnMut(
            &WorthQueryAdvancementPhase<'_>,
            WorthQueryAdmittedApplicationOperation<
                BankSchema,
                RevokeEstateCapabilityOperation,
                EstateAction,
                EstateCase,
            >,
        ) -> Result<
            WorthQueryCapabilityRevocationProgram<
                BankSchema,
                RevokeEstateCapabilityOperation,
                EstateAction,
                EstateCase,
            >,
            BankEstateProgressionDenial,
        > + 'runtime,
>;
/// Capture refused before an owner was minted; all original caller custody is returned.
pub type BankDelegationCaptureFailure<'runtime> = (
    EstateAction,
    BankIdempotencyKey,
    BankDelegationPreparation<'runtime>,
    WorthQueryApplicationCapabilityDelegationDenial<BankEstateProgressionDenial>,
);
/// Capture refused before an owner was minted; all original caller custody is returned.
pub type BankRevocationCaptureFailure<'runtime> = (
    EstateAction,
    BankIdempotencyKey,
    BankRevocationPreparation<'runtime>,
    WorthQueryApplicationCapabilityRevocationDenial<BankEstateProgressionDenial>,
);
