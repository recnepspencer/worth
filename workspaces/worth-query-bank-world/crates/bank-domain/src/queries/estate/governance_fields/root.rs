use worth_query_decl::facade::{
    application_query::ApplicationQueryResultFieldRef,
    application_schema::{EqualityPredicate, NoApplicationUnit, ReadOnly, ReadWrite},
};

use crate::{
    estate::{EstateCaseId, EstateWorkflowStage},
    model::{BankPrincipalId, EmployeeAssignmentId, EmployeeRole},
    schema::{
        AssignmentRole, BankSchema, EmployeeAssignment, EmployeeAssignmentIdentityField,
        EmployeeScope, EstateCase, EstateCaseIdentityField, EstateCaseRecord,
        EstateWorkflowStageField, Principal, PrincipalIdentity, PrincipalIdentityField,
    },
};

use super::super::governance::EstateGovernanceQuery;

pub(in crate::queries::estate) struct EstateIdSlot;
worth_query_decl::facade::worth_query_portable_type!(EstateIdSlot => "EstateIdSlot");
pub(in crate::queries::estate) struct EstateStageSlot;
worth_query_decl::facade::worth_query_portable_type!(EstateStageSlot => "EstateStageSlot");
pub(in crate::queries::estate) struct BeneficiarySlot;
worth_query_decl::facade::worth_query_portable_type!(BeneficiarySlot => "BeneficiarySlot");
pub(in crate::queries::estate) struct AssignmentIdSlot;
worth_query_decl::facade::worth_query_portable_type!(AssignmentIdSlot => "AssignmentIdSlot");
pub(in crate::queries::estate) struct AssignmentRoleSlot;
worth_query_decl::facade::worth_query_portable_type!(AssignmentRoleSlot => "AssignmentRoleSlot");
pub(in crate::queries::estate) struct AssignmentPrincipalSlot;
worth_query_decl::facade::worth_query_portable_type!(AssignmentPrincipalSlot => "AssignmentPrincipalSlot");

macro_rules! selector {
    ($name:ident, $slot:ty, $entity:ty, $aspect:ty, $field:ty, $value:ty, $write:ty, $alias:literal) => {
        pub(in crate::queries::estate) fn $name() -> ApplicationQueryResultFieldRef<
            EstateGovernanceQuery,
            $slot,
            BankSchema,
            $entity,
            $aspect,
            $field,
            $value,
            $write,
            EqualityPredicate,
            NoApplicationUnit,
        > {
            ApplicationQueryResultFieldRef::new($alias, <$field>::reference())
        }
    };
}

selector!(
    estate_id,
    EstateIdSlot,
    EstateCase,
    EstateCaseRecord,
    EstateCaseIdentityField,
    EstateCaseId,
    ReadOnly,
    "estate"
);
selector!(
    estate_stage,
    EstateStageSlot,
    EstateCase,
    EstateCaseRecord,
    EstateWorkflowStageField,
    EstateWorkflowStage,
    ReadWrite,
    "stage"
);
selector!(
    assignment_id,
    AssignmentIdSlot,
    EmployeeAssignment,
    EmployeeScope,
    EmployeeAssignmentIdentityField,
    EmployeeAssignmentId,
    ReadOnly,
    "assignment"
);
selector!(
    assignment_role,
    AssignmentRoleSlot,
    EmployeeAssignment,
    EmployeeScope,
    AssignmentRole,
    EmployeeRole,
    ReadWrite,
    "role"
);

macro_rules! principal_selector {
    ($name:ident, $slot:ty, $alias:literal) => {
        selector!(
            $name,
            $slot,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityField,
            BankPrincipalId,
            ReadOnly,
            $alias
        );
    };
}

principal_selector!(beneficiary, BeneficiarySlot, "beneficiary");
principal_selector!(
    assignment_principal_identity,
    AssignmentPrincipalSlot,
    "principal"
);
