pub mod queries {
    pub use bank_domain::queries::{
        account_authorized_users, account_detail, account_summary, accounts, current_accounts,
        estate_case, estate_customer_identity, estate_emergency_access_activity,
        estate_emergency_account_details, estate_governance_context, estate_legal_compliance,
        estate_mandatory_reviews, institution_audit, payment, pending_payments,
    };
}
