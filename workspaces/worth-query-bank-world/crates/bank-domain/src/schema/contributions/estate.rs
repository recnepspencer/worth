use crate::authorization::{
    install_estate_ability_policies, ViewEstateCase, ViewEstateLegalCompliance,
    ViewEstateMandatoryReview,
};

use super::super::{estate::install_estate_world, BankSchema};

use worth_query_decl::facade::application_schema::{fragment::*, *};
pub struct BankEstate;
impl ApplicationSchemaContribution<BankSchema> for BankEstate {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.bank.estate.v1");
    type Members = (
        crate::schema::EstateRecords,
        Contracts,
        crate::schema::RestoreRecordedEstateFreeze,
    );
}
impl BankEstate {
    pub const fn reference() -> ApplicationSchemaContributionRef<BankSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<BankSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<BankSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok({
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
                .application_query_binding::<crate::queries::EstateMandatoryReviewQueryBinding>()
                .application_mutation_binding::<crate::schema::NotifyEstateDeathMutationBinding>()
                .application_mutation_binding::<crate::schema::RetransmitEstateDeathNoticeMutationBinding>()
                .application_mutation_binding::<crate::schema::FreezeEstateAccountMutationBinding>()
                .application_mutation_binding::<crate::schema::OpenEstateCaseMutationBinding>()
                .application_mutation_binding::<crate::schema::RecognizeEstateExecutorMutationBinding>()
                .application_mutation_binding::<crate::schema::ReleaseEstateMutationBinding>()
                .application_mutation_binding::<crate::schema::DisburseEstateMutationBinding>();
            install_estate_ability_policies(schema)
        })
    }
}
