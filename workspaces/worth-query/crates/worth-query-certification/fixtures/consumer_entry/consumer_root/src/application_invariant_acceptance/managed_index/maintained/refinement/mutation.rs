//! Exact expectations use the same declared rename operation and field handler.
use super::super::{mutation as ordinary, schema::*};
use super::{ExactQuery, FindExact};
use worth_query_decl::facade::{
    application_operation::*, application_query::*, application_schema::*,
};

pub struct Binding;
impl ApplicationMutationAdmissionBinding<Catalog> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Catalog> for Binding {
    type Input = ordinary::Rename;
    type InputBinding = ordinary::InputBinding;
    type Result = ordinary::Renamed;
    type ResultBinding = ordinary::ResultBinding;
    type IdempotencyKey = u64;
    type Operation = ordinary::RenameItem;
    type Decision = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::Decision;
    type Denial = ordinary::Denial;
    type DenialBinding = ordinary::DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = <ordinary::Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding;
    type PrincipalBinding = CatalogPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<ExactQuery>;
    const IDENTITY: &'static str = "maintained-catalog.rename-exact.binding.v1";
    const HANDLER_IDENTITY: &'static str = "maintained-catalog.rename-exact.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "maintained-catalog.rename-exact.key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> DirectoryIdentity {
        DirectoryIdentity::at("DirectoryIdentity")
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
    fn expected_source_parameters(
        input: &ordinary::Rename,
    ) -> Result<Option<ApplicationQueryParameterSet<ExactQuery>>, ApplicationValueEncodeDenial>
    {
        Ok(Some(
            <FindExact as ApplicationQueryIntent<Catalog>>::parameters(&FindExact {
                directory: input.directory.clone(),
                name: input.prefix.clone(),
            }),
        ))
    }
}
#[derive(Clone, Debug)]
pub struct RenameExact(pub ordinary::Rename);
impl ApplicationMutationIntent<Catalog> for RenameExact {
    type Binding = Binding;
    fn input(&self) -> &ordinary::Rename {
        &self.0
    }
    fn scope_binding(&self) -> <Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
        ApplicationMutationFieldScope::new(
            DirectoryIdentity::at("DirectoryIdentity"),
            self.0.directory.clone(),
        )
    }
}
