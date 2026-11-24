//! Durable mutation inputs, bindings, intents, and schema registrations.

use worth_query_decl::facade::{
    application_operation::*, application_schema::*,
    authentication::WorthQueryExternalPrincipalIdentity, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_emits, worth_query_operation_links,
    worth_query_operation_reads, worth_query_operation_writes, worth_query_portable_type,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::{
    primary_graph::WorthQueryInvariantMutationTarget, shared_access::SharedAccessDenial,
};

use super::capabilities::SharedPrincipalBinding;
use super::query::ProjectUpdatedEffect;
use super::schema::{
    ExternalMapping, GrantActiveField, GrantIdField, GrantNotBeforeField, GrantResource, Grantee,
    Grantor, Principal, PrincipalIdField, Project, ProjectGrant, ProjectKeyField,
    ProjectVersionField, SharedProjectSchema,
};

pub const ISSUER: &str = "https://daymark.local";
pub const OWNER_SUBJECT: &str = "owner";
pub const ALICE_SUBJECT: &str = "alice";
pub const BOB_SUBJECT: &str = "bob";
pub const OWNER_PRINCIPAL_ID: u64 = 1;
pub const ALICE_PRINCIPAL_ID: u64 = 2;
pub const BOB_PRINCIPAL_ID: u64 = 3;

pub fn owner_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(ISSUER, OWNER_SUBJECT).unwrap()
}

pub fn alice_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(ISSUER, ALICE_SUBJECT).unwrap()
}

pub fn bob_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(ISSUER, BOB_SUBJECT).unwrap()
}

pub fn principal_id_for(
    identity: &WorthQueryExternalPrincipalIdentity,
) -> Result<u64, SharedAccessDenial> {
    match identity.subject() {
        OWNER_SUBJECT => Ok(OWNER_PRINCIPAL_ID),
        ALICE_SUBJECT => Ok(ALICE_PRINCIPAL_ID),
        BOB_SUBJECT => Ok(BOB_PRINCIPAL_ID),
        _ => Err(SharedAccessDenial::GrantMutationRefused(
            "unknown collaborator principal".to_owned(),
        )),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectId(pub String);
impl ProjectId {
    pub fn project1() -> Self {
        Self("Project1".to_owned())
    }
    pub fn project2() -> Self {
        Self("Project2".to_owned())
    }
}

worth_query_structured_value_binding!(pub UnitDenialBinding for () {
    identity: "worth.query.court.shared_access.unit_denial.v1"
});
worth_query_structured_value_binding!(pub U64ResultBinding for u64 {
    identity: "worth.query.court.shared_access.u64_result_binding.v1"
});

pub type ProjectFieldScope = ApplicationMutationFieldScope<ProjectKeyField>;
pub type GrantFieldScope = ApplicationMutationFieldScope<GrantIdField>;

// --- 1. WriteProjectVersion ---
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WriteProjectInput {
    pub project_key: String,
    pub next_version: u64,
}
worth_query_portable_type!(WriteProjectInput => "worth.query.court.shared_access.write_project_input.v1");
worth_query_structured_value_binding!(pub WriteProjectInputBinding for WriteProjectInput {
    identity: "worth.query.court.shared_access.write_project_input_binding.v1"
});
worth_query_operation!(pub WriteProjectVersion for SharedProjectSchema, input WriteProjectInputBinding);
worth_query_operation_reads!(WriteProjectVersion => [ProjectKeyField, ProjectVersionField]);
worth_query_operation_writes!(WriteProjectVersion => [ProjectVersionField]);
worth_query_operation_emits!(WriteProjectVersion => [ProjectUpdatedEffect]);

pub struct WriteProjectVersionBinding;
impl ApplicationMutationAdmissionBinding<SharedProjectSchema> for WriteProjectVersionBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<SharedProjectSchema> for WriteProjectVersionBinding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = WriteProjectInput;
    type InputBinding = WriteProjectInputBinding;
    type Result = u64;
    type ResultBinding = U64ResultBinding;
    type IdempotencyKey = u64;
    type Operation = WriteProjectVersion;
    type Decision = WorthQueryInvariantMutationTarget<SharedProjectSchema, Project>;
    type Denial = ();
    type DenialBinding = UnitDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ProjectFieldScope;
    type PrincipalBinding = SharedPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.court.shared_access.write_project.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.court.shared_access.write_project_handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.court.shared_access.write_project_cmd.v1";
    fn scope_field() -> ProjectKeyField {
        ProjectKeyField::at("ProjectKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        SharedProjectSchema,
        SharedPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        SharedPrincipalBinding::reference()
    }
}

#[derive(Clone, Debug)]
pub struct WriteProjectIntent(pub WriteProjectInput);
impl ApplicationMutationIntent<SharedProjectSchema> for WriteProjectIntent {
    type Binding = WriteProjectVersionBinding;
    fn input(&self) -> &WriteProjectInput {
        &self.0
    }
    fn scope_binding(&self) -> ProjectFieldScope {
        ProjectFieldScope::new(
            ProjectKeyField::at("ProjectKey"),
            self.0.project_key.clone(),
        )
    }
}

// --- 2. GrantProjectAccess ---
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GrantProjectInput {
    pub grant_id: String,
    pub project_key: String,
    pub grantee_principal_id: u64,
    pub mode: String,
}
worth_query_portable_type!(GrantProjectInput => "worth.query.court.shared_access.grant_input.v1");
worth_query_structured_value_binding!(pub GrantProjectInputBinding for GrantProjectInput {
    identity: "worth.query.court.shared_access.grant_input_binding.v1"
});
worth_query_operation!(pub GrantProjectAccess for SharedProjectSchema, input GrantProjectInputBinding);
worth_query_operation_reads!(GrantProjectAccess => [ProjectKeyField, PrincipalIdField]);
worth_query_operation_creates!(GrantProjectAccess => [ProjectGrant]);
worth_query_operation_writes!(GrantProjectAccess => [
    GrantIdField,
    GrantActiveField,
    GrantNotBeforeField
]);
worth_query_operation_links!(GrantProjectAccess => [Grantor, Grantee, GrantResource]);

pub struct GrantProjectDecision {
    pub owner: WorthQueryInvariantMutationTarget<SharedProjectSchema, Principal>,
    pub grantee: WorthQueryInvariantMutationTarget<SharedProjectSchema, Principal>,
    pub project: WorthQueryInvariantMutationTarget<SharedProjectSchema, Project>,
}

pub struct GrantProjectAccessBinding;
impl ApplicationMutationAdmissionBinding<SharedProjectSchema> for GrantProjectAccessBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<SharedProjectSchema> for GrantProjectAccessBinding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = GrantProjectInput;
    type InputBinding = GrantProjectInputBinding;
    type Result = u64;
    type ResultBinding = U64ResultBinding;
    type IdempotencyKey = u64;
    type Operation = GrantProjectAccess;
    type Decision = GrantProjectDecision;
    type Denial = ();
    type DenialBinding = UnitDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ProjectFieldScope;
    type PrincipalBinding = SharedPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.court.shared_access.grant_access.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.court.shared_access.grant_access_handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.court.shared_access.grant_access_cmd.v1";
    fn scope_field() -> ProjectKeyField {
        ProjectKeyField::at("ProjectKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        SharedProjectSchema,
        SharedPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        SharedPrincipalBinding::reference()
    }
}

#[derive(Clone, Debug)]
pub struct GrantProjectIntent(pub GrantProjectInput);
impl ApplicationMutationIntent<SharedProjectSchema> for GrantProjectIntent {
    type Binding = GrantProjectAccessBinding;
    fn input(&self) -> &GrantProjectInput {
        &self.0
    }
    fn scope_binding(&self) -> ProjectFieldScope {
        ProjectFieldScope::new(
            ProjectKeyField::at("ProjectKey"),
            self.0.project_key.clone(),
        )
    }
}

// --- 3. RevokeProjectAccess ---
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RevokeProjectInput {
    pub grant_id: String,
}
worth_query_portable_type!(RevokeProjectInput => "worth.query.court.shared_access.revoke_input.v1");
worth_query_structured_value_binding!(pub RevokeProjectInputBinding for RevokeProjectInput {
    identity: "worth.query.court.shared_access.revoke_input_binding.v1"
});
worth_query_operation!(pub RevokeProjectAccess for SharedProjectSchema, input RevokeProjectInputBinding);
worth_query_operation_reads!(RevokeProjectAccess => [GrantIdField, GrantActiveField]);
worth_query_operation_writes!(RevokeProjectAccess => [GrantActiveField]);

pub struct RevokeProjectAccessBinding;
impl ApplicationMutationAdmissionBinding<SharedProjectSchema> for RevokeProjectAccessBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<SharedProjectSchema> for RevokeProjectAccessBinding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = RevokeProjectInput;
    type InputBinding = RevokeProjectInputBinding;
    type Result = u64;
    type ResultBinding = U64ResultBinding;
    type IdempotencyKey = u64;
    type Operation = RevokeProjectAccess;
    type Decision = WorthQueryInvariantMutationTarget<SharedProjectSchema, ProjectGrant>;
    type Denial = ();
    type DenialBinding = UnitDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = GrantFieldScope;
    type PrincipalBinding = SharedPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.court.shared_access.revoke_access.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.court.shared_access.revoke_access_handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.court.shared_access.revoke_access_cmd.v1";
    fn scope_field() -> GrantIdField {
        GrantIdField::at("GrantId")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        SharedProjectSchema,
        SharedPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        SharedPrincipalBinding::reference()
    }
}

#[derive(Clone, Debug)]
pub struct RevokeProjectIntent(pub RevokeProjectInput);
impl ApplicationMutationIntent<SharedProjectSchema> for RevokeProjectIntent {
    type Binding = RevokeProjectAccessBinding;
    fn input(&self) -> &RevokeProjectInput {
        &self.0
    }
    fn scope_binding(&self) -> GrantFieldScope {
        GrantFieldScope::new(GrantIdField::at("GrantId"), self.0.grant_id.clone())
    }
}
