//! Expected contract loci use the independently declared record selectors.
pub fn native_aspect(entity: &str, meaning: &str) -> &'static str {
    match (entity, meaning) {
        ("Account", "AccountProfile") => bank_domain::schema::AccountProfile::reference().name(),
        ("Account", "AccountState") => bank_domain::schema::AccountState::reference().name(),
        ("Account", "Identity") => bank_domain::schema::Identity::reference().name(),
        ("AccountAuthorization", "AuthorizationIdentity") => {
            bank_domain::schema::AuthorizationIdentity::reference().name()
        }
        ("AccountAuthorization", "AuthorizationScope") => {
            bank_domain::schema::AuthorizationScope::reference().name()
        }
        ("Branch", "BranchIdentity") => bank_domain::schema::BranchIdentity::reference().name(),
        ("Business", "BusinessIdentity") => {
            bank_domain::schema::BusinessIdentity::reference().name()
        }
        ("CapabilityGrant", "CapabilityGrantRecord") => {
            bank_domain::schema::CapabilityGrantRecord::reference().name()
        }
        ("DeathNotice", "DeathNoticeRecord") => {
            bank_domain::schema::DeathNoticeRecord::reference().name()
        }
        ("EmergencyAccess", "EmergencyAccessRecord") => {
            bank_domain::schema::EmergencyAccessRecord::reference().name()
        }
        ("EmployeeAssignment", "EmployeeScope") => {
            bank_domain::schema::EmployeeScope::reference().name()
        }
        ("EstateCase", "EstateCaseRecord") => {
            bank_domain::schema::EstateCaseRecord::reference().name()
        }
        ("ExternalPrincipalMapping", "ExternalPrincipalIdentity") => {
            bank_domain::schema::ExternalPrincipalIdentity::reference().name()
        }
        ("Institution", "InstitutionIdentity") => {
            bank_domain::schema::InstitutionIdentity::reference().name()
        }
        ("JournalEntry", "JournalIdentity") => {
            bank_domain::schema::JournalIdentity::reference().name()
        }
        ("JournalEntry", "JournalState") => bank_domain::schema::JournalState::reference().name(),
        ("LegalAuthority", "LegalAuthorityRecord") => {
            bank_domain::schema::LegalAuthorityRecord::reference().name()
        }
        ("MandatoryReview", "MandatoryReviewRecord") => {
            bank_domain::schema::MandatoryReviewRecord::reference().name()
        }
        ("PaymentIntent", "PaymentIdentity") => {
            bank_domain::schema::PaymentIdentity::reference().name()
        }
        ("PaymentIntent", "PaymentState") => bank_domain::schema::PaymentState::reference().name(),
        ("PaymentIntent", "PaymentValue") => bank_domain::schema::PaymentValue::reference().name(),
        ("Posting", "PostingIdentity") => bank_domain::schema::PostingIdentity::reference().name(),
        ("Posting", "PostingValue") => bank_domain::schema::PostingValue::reference().name(),
        ("Principal", "PrincipalIdentity") => {
            bank_domain::schema::PrincipalIdentity::reference().name()
        }
        ("ApprovedBusinessPaymentGrant", "ApprovedBusinessPaymentGrantFacts") => {
            bank_domain::schema::ApprovedBusinessPaymentGrantFacts::reference().name()
        }
        ("ProgramDeploymentGrant", "ProgramDeploymentGrantFacts") => {
            bank_domain::schema::ProgramDeploymentGrantFacts::reference().name()
        }
        ("ProgramDeploymentScope", "ProgramDeploymentScopeFacts") => {
            bank_domain::schema::ProgramDeploymentScopeFacts::reference().name()
        }
        _ => panic!("the fixture must select a declared bank aspect"),
    }
}
