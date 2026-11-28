//! Exact estate source occurrences; fresh native capability admission is binding-selected.
use super::composition::BankEstateFeature;
use bank_domain::schema::*;
use worth_query_host::facade::declaration::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationRootComposition,
    ApplicationSourceCompletion,
};
/// Source completion for the exact NotifyEstateDeath occurrence.
pub struct DeathNotificationDelivery;
impl ApplicationActionDelivery<BankSchema, NotifyEstateDeathMutationBinding>
    for DeathNotificationDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.estate.deathnotification.completion.v1";
}
/// The installed NotifyEstateDeath action.
pub type DeathNotificationAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankEstateFeature,
    NotifyEstateDeathMutationBinding,
    DeathNotificationDelivery,
>;
/// Source completion for the exact FreezeEstateAccount occurrence.
pub struct EstateAccountFreezeDelivery;
impl ApplicationActionDelivery<BankSchema, FreezeEstateAccountMutationBinding>
    for EstateAccountFreezeDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.estate.estateaccountfreeze.completion.v1";
}
/// The installed FreezeEstateAccount action.
pub type EstateAccountFreezeAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankEstateFeature,
    FreezeEstateAccountMutationBinding,
    EstateAccountFreezeDelivery,
>;
/// Source completion for the exact OpenEstateCase occurrence.
pub struct EstateCaseOpeningDelivery;
impl ApplicationActionDelivery<BankSchema, OpenEstateCaseMutationBinding>
    for EstateCaseOpeningDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.estate.estatecaseopening.completion.v1";
}
/// The installed OpenEstateCase action.
pub type EstateCaseOpeningAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankEstateFeature,
    OpenEstateCaseMutationBinding,
    EstateCaseOpeningDelivery,
>;
/// Source completion for the exact RecognizeEstateExecutor occurrence.
pub struct ExecutorRecognitionDelivery;
impl ApplicationActionDelivery<BankSchema, RecognizeEstateExecutorMutationBinding>
    for ExecutorRecognitionDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.estate.executorrecognition.completion.v1";
}
/// The installed RecognizeEstateExecutor action.
pub type ExecutorRecognitionAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankEstateFeature,
    RecognizeEstateExecutorMutationBinding,
    ExecutorRecognitionDelivery,
>;
/// Source completion for the exact ReleaseEstate occurrence.
pub struct EstateReleaseDelivery;
impl ApplicationActionDelivery<BankSchema, ReleaseEstateMutationBinding> for EstateReleaseDelivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.estate.estaterelease.completion.v1";
}
/// The installed ReleaseEstate action.
pub type EstateReleaseAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankEstateFeature,
    ReleaseEstateMutationBinding,
    EstateReleaseDelivery,
>;
/// Source completion for the exact DisburseEstate occurrence.
pub struct EstateDisbursementDelivery;
impl ApplicationActionDelivery<BankSchema, DisburseEstateMutationBinding>
    for EstateDisbursementDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.estate.estatedisbursement.completion.v1";
}
/// The installed DisburseEstate action.
pub type EstateDisbursementAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankEstateFeature,
    DisburseEstateMutationBinding,
    EstateDisbursementDelivery,
>;
/// Source completion for the exact RetransmitEstateDeathNotice occurrence.
pub struct DeathNoticeRetransmissionDelivery;
impl ApplicationActionDelivery<BankSchema, RetransmitEstateDeathNoticeMutationBinding>
    for DeathNoticeRetransmissionDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "bank.estate.deathnoticeretransmission.completion.v1";
}
/// The installed RetransmitEstateDeathNotice action.
pub type DeathNoticeRetransmissionAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankEstateFeature,
    RetransmitEstateDeathNoticeMutationBinding,
    DeathNoticeRetransmissionDelivery,
>;
