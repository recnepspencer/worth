mod account_activity;
mod account_authorized_users;
mod account_detail;
mod account_discovery;
mod account_summary;
mod account_summary_projection;
mod estate;
mod institution_audit;
mod payment_detail;
mod payment_summary_projection;
mod pending_payments;

worth_query_decl::facade::worth_query_structured_value_binding!(pub(crate) UnitQueryResultBinding for () { identity: "worth.rust.unit" });

pub use account_activity::{
    account_activity, account_activity_definition, AccountActivityLiveCause, AccountActivityQuery,
    AccountActivityQueryBinding, AccountActivityQueryParameters, AccountActivityQueryResult,
    AccountActivityRequest,
};
pub use account_authorized_users::{
    account_authorized_users, account_authorized_users_definition, AccountAuthorizedUsersQuery,
    AccountAuthorizedUsersQueryBinding, AccountAuthorizedUsersQueryParameters,
    AccountAuthorizedUsersQueryResult, AccountAuthorizedUsersRequest,
};
pub use account_detail::{
    account_detail, account_detail_definition, AccountDetailQuery, AccountDetailQueryBinding,
    AccountDetailQueryParameters, AccountDetailRequest,
};
pub use account_discovery::{
    account_discovery_definition, accounts, AccountDiscoveryQuery, AccountDiscoveryQueryBinding,
    AccountDiscoveryQueryParameters, AccountDiscoveryRequest,
};
pub use account_summary::{
    account_summary, account_summary_definition, AccountSummaryQuery, AccountSummaryQueryBinding,
    AccountSummaryQueryParameters, AccountSummaryRequest,
};
pub use estate::{
    estate_case, estate_case_overview_definition, estate_customer_disclosure_definition,
    estate_customer_identity, estate_emergency_access_activity,
    estate_emergency_access_activity_definition, estate_emergency_account_details,
    estate_emergency_account_details_definition, estate_governance_context,
    estate_governance_definition, estate_legal_compliance, estate_legal_compliance_definition,
    estate_mandatory_review_definition, estate_mandatory_reviews, EstateCaseOverviewQuery,
    EstateCaseOverviewQueryBinding, EstateCaseOverviewQueryParameters, EstateCaseOverviewRequest,
    EstateCustomerDisclosure, EstateCustomerDisclosureQuery, EstateCustomerDisclosureQueryBinding,
    EstateCustomerDisclosureQueryParameters, EstateCustomerDisclosureRequest,
    EstateEmergencyAccessActivity, EstateEmergencyAccessActivityItem,
    EstateEmergencyAccessActivityLiveCause, EstateEmergencyAccessActivityQuery,
    EstateEmergencyAccessActivityQueryBinding, EstateEmergencyAccessActivityQueryParameters,
    EstateEmergencyAccessActivityRequest, EstateEmergencyAccountDetails,
    EstateEmergencyAccountDetailsQuery, EstateEmergencyAccountDetailsQueryBinding,
    EstateEmergencyAccountDetailsQueryParameters, EstateEmergencyAccountDetailsRequest,
    EstateGovernanceQuery, EstateGovernanceQueryBinding, EstateGovernanceQueryParameters,
    EstateGovernanceRequest, EstateLegalComplianceQuery, EstateLegalComplianceQueryBinding,
    EstateLegalComplianceQueryParameters, EstateLegalComplianceRequest,
    EstateLegalComplianceResult, EstateMandatoryReviewQuery, EstateMandatoryReviewQueryBinding,
    EstateMandatoryReviewQueryParameters, EstateMandatoryReviewRequest,
    EstateMandatoryReviewResult,
};
pub use institution_audit::{
    institution_audit, institution_audit_definition, InstitutionAuditQuery,
    InstitutionAuditQueryBinding, InstitutionAuditQueryParameters, InstitutionAuditRequest,
};
pub use payment_detail::{
    payment, payment_detail_definition, PaymentDetailQuery, PaymentDetailQueryBinding,
    PaymentDetailQueryParameters, PaymentDetailRequest,
};
pub use pending_payments::{
    pending_payments, pending_payments_definition, PendingPaymentsQuery,
    PendingPaymentsQueryBinding, PendingPaymentsQueryParameters, PendingPaymentsRequest,
};
