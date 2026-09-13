use worth_query_decl::facade::worth_query_application_contribution;

use crate::authorization::{
    install_estate_ability_policies, ViewEstateCase, ViewEstateLegalCompliance,
    ViewEstateMandatoryReview,
};

use super::super::{estate::install_estate_world, BankSchema};

worth_query_application_contribution! {
    pub contribution BankEstate in BankSchema {
        identity: "worth.bank.estate.v1",
        members: |schema| {
            let schema = install_estate_world(schema)
                .ability(ViewEstateCase::reference())
                .ability(ViewEstateLegalCompliance::reference())
                .ability(ViewEstateMandatoryReview::reference())
                .application_query(crate::queries::estate_case_overview_definition())
                .application_query_binding::<crate::queries::EstateCaseOverviewQueryBinding>()
                .application_query(crate::queries::estate_customer_disclosure_definition())
                .application_query_binding::<crate::queries::EstateCustomerDisclosureQueryBinding>()
                .application_query(crate::queries::estate_emergency_account_details_definition())
                .application_query_binding::<crate::queries::EstateEmergencyAccountDetailsQueryBinding>()
                .application_query(crate::queries::estate_emergency_access_activity_definition())
                .application_query_binding::<crate::queries::EstateEmergencyAccessActivityQueryBinding>()
                .application_query(crate::queries::estate_governance_definition())
                .application_query_binding::<crate::queries::EstateGovernanceQueryBinding>()
                .application_query(crate::queries::estate_legal_compliance_definition())
                .application_query_binding::<crate::queries::EstateLegalComplianceQueryBinding>()
                .application_query(crate::queries::estate_mandatory_review_definition())
                .application_query_binding::<crate::queries::EstateMandatoryReviewQueryBinding>();
            install_estate_ability_policies(schema)
        }
    }
}
