pub use bank_domain::{
    estate::{
        BranchId, CapabilityGrantId, CapabilityValidity, DelegationLimit, EstateAction,
        EstateCapabilityDelegationRequest, EstateCapabilityOperation, EstateCapabilityPurpose,
        EstateCapabilityScope, EstateCaseId, EstateMoment, EstateWorkflowStage,
    },
    model::{BankPrincipalId, InstitutionId},
    schema::{
        BankSchema, BranchIdBinding, DelegateEstateCapability, DelegateEstateCapabilityOperation,
        InstitutionIdBinding,
    },
};
pub use worth_query_host::facade::declaration::{
    application_capability::ApplicationCapabilityDelegationRequest,
    application_schema::ApplicationScalarValueBinding,
};
pub use worth_query_host::facade::domain::WorthQueryInstallationRuntimeIdentity;

pub use super::{installed_bank, installed_program_targets, InstalledProgramTarget};

pub fn write(field: &str) -> InstalledProgramTarget {
    InstalledProgramTarget::Write {
        entity: "CapabilityGrant".to_owned(),
        aspect: "CapabilityGrantRecord".to_owned(),
        path: worth_foundational::facade::CanonicalFieldPath::single(
            worth_foundational::facade::FieldKey::new(field).unwrap(),
        ),
    }
}

pub fn link(relation: &str, from: &str, to: &str) -> InstalledProgramTarget {
    InstalledProgramTarget::Link {
        relation: relation.to_owned(),
        from: from.to_owned(),
        to: to.to_owned(),
    }
}
