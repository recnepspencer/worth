//! Distinct fixed action on the original Catalog feature and Request operation.
use super::super::super::schema::*;
use super::super::source::operation as ordinary;
use worth_query_decl::facade::{application_operation::*, application_schema::*};
pub struct Binding;
impl ApplicationMutationAdmissionBinding<Catalog> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Catalog> for Binding {
    type Input = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::Input;
    type InputBinding = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::InputBinding;
    type Result = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::Result;
    type ResultBinding = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::ResultBinding;
    type IdempotencyKey =
        <ordinary::Binding as ApplicationMutationBinding<Catalog>>::IdempotencyKey;
    type Operation = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::Operation;
    type Decision = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::Decision;
    type Denial = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::Denial;
    type DenialBinding = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::DenialBinding;
    type Output = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::Output;
    type ScopeBinding = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding;
    type PrincipalBinding =
        <ordinary::Binding as ApplicationMutationBinding<Catalog>>::PrincipalBinding;
    type Mapping = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::Mapping;
    type Principal = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::Principal;
    type PrincipalIdentity =
        <ordinary::Binding as ApplicationMutationBinding<Catalog>>::PrincipalIdentity;
    type PrincipalIdentityBinding =
        <ordinary::Binding as ApplicationMutationBinding<Catalog>>::PrincipalIdentityBinding;
    type SourceExpectation =
        <ordinary::Binding as ApplicationMutationBinding<Catalog>>::SourceExpectation;
    const IDENTITY: &'static str = "maintained-generated.indexed.request-binding.v1";
    const HANDLER_IDENTITY: &'static str = "maintained-generated.indexed.request-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "maintained-generated.indexed.request-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        Catalog,
        Directory,
        DirectoryFacts,
        DirectoryIdentity,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DirectoryIdentity::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Catalog,
        CatalogPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        CatalogPrincipal::reference()
    }
}
#[derive(Clone, Debug)]
pub struct Request(pub ordinary::RequestItem);
impl ApplicationMutationIntent<Catalog> for Request {
    type Binding = Binding;
    fn input(&self) -> &ordinary::RequestItem {
        &self.0
    }
    fn scope_binding(&self) -> <Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
        ApplicationMutationFieldScope::new(DirectoryIdentity::reference(), self.0.directory.clone())
    }
}
