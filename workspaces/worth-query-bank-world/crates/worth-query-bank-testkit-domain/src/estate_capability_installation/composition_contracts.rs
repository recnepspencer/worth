pub use bank_domain::{
    estate::{CapabilityGrantStatus, RestrictedBankField},
    model::EmployeeRole,
    schema::{
        ApproveEstateEmergencyAccessCapability, ApproveEstateEmergencyAccessOperation,
        CapabilityGrantStatusBinding, CompleteEstateMandatoryReviewCapability,
        CompleteEstateMandatoryReviewOperation, EmployeeRoleBinding,
        RecognizeEstateExecutorCapability, RecognizeEstateExecutorOperation,
        ReleaseEstateCapability, ReleaseEstateOperation, RequestEstateEmergencyAccessCapability,
        RequestEstateEmergencyAccessOperation, RestrictedBankFieldBinding,
        ViewEstateAdministrationCapability, ViewRestrictedEstateOperation,
    },
};
pub use worth_query_host::facade::{
    declaration::{
        application_capability::{
            ApplicationCapabilityDisclosureRule, ApplicationCapabilityGraphRule,
        },
        application_schema::ApplicationScalarValueBinding,
    },
    domain::WorthQueryInstallationRuntimeIdentity,
};

pub use super::installed_bank;

pub fn path_relations(rule: &ApplicationCapabilityGraphRule) -> Vec<&str> {
    let mut relations = all_clauses(rule)
        .iter()
        .flat_map(|clause| clause.path().traversals())
        .map(|traversal| traversal.relation())
        .collect::<Vec<_>>();
    relations.sort_unstable();
    relations.dedup();
    relations
}

pub fn all_clauses(
    rule: &ApplicationCapabilityGraphRule,
) -> Vec<&worth_query_host::facade::declaration::application_capability::ApplicationCapabilityGraphClause>
{
    rule.requirements()
        .iter()
        .flat_map(|requirement| requirement.clauses())
        .collect()
}

pub fn requirement_widths(rule: &ApplicationCapabilityGraphRule) -> Vec<usize> {
    let mut widths = rule
        .requirements()
        .iter()
        .map(|requirement| requirement.clauses().len())
        .collect::<Vec<_>>();
    widths.sort_unstable();
    widths
}

pub fn anchor_relations(
    clause: &worth_query_host::facade::declaration::application_capability::ApplicationCapabilityGraphClause,
) -> Vec<&str> {
    clause
        .context_anchors()
        .iter()
        .map(|anchor| anchor.relation().relation())
        .collect()
}

pub fn expected_values(
    fields: impl IntoIterator<Item = RestrictedBankField>,
) -> Vec<worth_foundational::facade::AspectValue> {
    let mut values = fields
        .into_iter()
        .map(|field| RestrictedBankFieldBinding::encode(&field).unwrap())
        .collect::<Vec<_>>();
    values.sort();
    values
}

pub fn expected_roles(
    roles: impl IntoIterator<Item = EmployeeRole>,
) -> Vec<worth_foundational::facade::AspectValue> {
    let mut values = roles
        .into_iter()
        .map(|role| EmployeeRoleBinding::encode(&role).unwrap())
        .collect::<Vec<_>>();
    values.sort();
    values
}
