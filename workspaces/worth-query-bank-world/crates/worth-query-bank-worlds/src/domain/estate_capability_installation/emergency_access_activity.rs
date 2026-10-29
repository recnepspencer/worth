pub use bank_domain::{
    estate::{EmergencyAccessStatus, RestrictedBankField},
    queries::EstateEmergencyAccessActivityQueryBinding,
    schema::{
        ApproveEstateEmergencyAccessCapability, ApproveEstateEmergencyAccessOperation,
        CompleteEstateMandatoryReviewCapability, CompleteEstateMandatoryReviewOperation,
        EmergencyAccessStatusBinding, EstateEmergencyAccessActivityEventBinding,
        RequestEstateEmergencyAccessCapability, RequestEstateEmergencyAccessOperation,
        RestrictedBankFieldBinding, RevokeEstateEmergencyAccessCapability,
        RevokeEstateEmergencyAccessOperation, ViewEstateAdministrationCapability,
        ViewEstateEmergencyProtectionCapability, ViewRestrictedEstateOperation,
    },
};
pub use worth_query_host::facade::{
    declaration::{
        application_capability::ApplicationCapabilityValidityTimeline,
        application_query::{
            ApplicationQueryDisclosurePosture, ApplicationQueryObservableInfluence,
            ApplicationQueryOrderingDirection, ApplicationQueryResultTraversalDirection,
        },
        application_schema::{ApplicationScalarValueBinding, ApplicationStructuredValueBinding},
    },
    domain::WorthQueryInstallationRuntimeIdentity,
};

pub use super::installed_bank;
pub use worth_query_host::facade::domain::WorthQueryOperationTouchScope;

pub fn assert_field_influences(
    disclosure: &worth_query_host::facade::declaration::application_query::ApplicationQueryDisclosureContract,
    field: &str,
    expected: &[ApplicationQueryObservableInfluence],
    expected_rule_count: usize,
) {
    let matching = disclosure
        .rules()
        .iter()
        .filter(|rule| {
            rule.selector()
                .field_contract()
                .is_some_and(|(_, _, candidate)| candidate == field)
        })
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), expected_rule_count, "field {field}");
    for rule in matching {
        assert_eq!(
            rule.influence().permitted().collect::<Vec<_>>(),
            expected,
            "field {field}"
        );
    }
}

pub fn relation_influences(
    disclosure: &worth_query_host::facade::declaration::application_query::ApplicationQueryDisclosureContract,
    relation: &str,
) -> Vec<ApplicationQueryObservableInfluence> {
    disclosure
        .rules()
        .iter()
        .find(|rule| {
            rule.selector()
                .relation_contract()
                .is_some_and(|(candidate, ..)| candidate == relation)
        })
        .expect("relation has one governed disclosure rule")
        .influence()
        .permitted()
        .collect()
}

pub fn disclosure_values<Schema, Capability, Operation, Input>(
    capability: &worth_query_host::facade::domain::WorthQueryInstalledApplicationCapability<
        Schema,
        Capability,
        Operation,
        Input,
    >,
) -> Vec<worth_foundational::facade::AspectValue>
where
    Schema: worth_query_host::facade::declaration::application_schema::ApplicationSchema,
{
    let worth_query_host::facade::declaration::application_capability::ApplicationCapabilityDisclosureRule::Permit(guards) =
        capability.contract().composition().propagation().disclosure()
    else {
        panic!("view capability must install an explicit disclosure matrix");
    };
    guards[0].requirements()[0].values().to_vec()
}
