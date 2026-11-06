//! Distinct runtime mutation binding over the original operation and output role.
use super::super::super::{query, schema::*};
use super::super::operation::{
    Decision, Denial, DenialBinding, Generate, GenerateItem, Generated, InputBinding, Outputs,
    ResultBinding,
};
use worth_query_decl::facade::{
    application_operation::*, application_query::*, application_schema::*,
};
pub struct Binding;
impl ApplicationMutationAdmissionBinding<Catalog> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Catalog> for Binding {
    type Input = Generate;
    type InputBinding = InputBinding;
    type Result = Generated;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = GenerateItem;
    type Decision = Decision;
    type Denial = Denial;
    type DenialBinding = DenialBinding;
    type Output = Outputs;
    type ScopeBinding = ApplicationMutationFieldScope<DirectoryIdentity>;
    type PrincipalBinding = CatalogPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<query::PrefixQuery>;
    const IDENTITY: &'static str = "maintained-generated.indexed.operation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "maintained-generated.indexed.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "maintained-generated.indexed.idempotency.v1";
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
        input: &Generate,
    ) -> Result<
        Option<ApplicationQueryParameterSet<query::PrefixQuery>>,
        ApplicationValueEncodeDenial,
    > {
        Ok(Some(<query::FindPrefix as ApplicationQueryIntent<
            Catalog,
        >>::parameters(&query::FindPrefix {
            directory: input.directory.clone(),
            prefix: "sh".into(),
        })))
    }
}

impl ApplicationMutationInputScope<Catalog, Binding> for Generate {
    fn scope_for_binding(&self) -> <Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
        ApplicationMutationFieldScope::new(
            DirectoryIdentity::at("DirectoryIdentity"),
            self.directory.clone(),
        )
    }
}
