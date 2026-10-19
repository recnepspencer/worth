//! Normal query-source mutation binding, with no effect callback or arithmetic.
use super::{
    actions::{DenialBinding, ResultBinding},
    path_value::{Bump, BumpInput, BumpRow},
    revision_query,
    schema::*,
};
use worth_query_decl::facade::{application_operation::*, application_schema::*};
use worth_query_host::facade::primary_graph::{
    WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial,
};
pub struct BumpBinding;
impl ApplicationMutationAdmissionBinding<Schema> for BumpBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Schema> for BumpBinding {
    type Input = BumpRow;
    type InputBinding = BumpInput;
    type Result = ();
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = Bump;
    type Decision = WorthQueryOperationBodyDecision<Schema, Row>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<
        Schema,
        Row,
        RowFacts,
        RowIdentity,
        String,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<revision_query::Query>;
    const IDENTITY: &'static str = "worth.query.path-value.binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.path-value.body.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.path-value.request.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(1, 0, 0, 0, 4, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(4096),
        );
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Row,
        RowFacts,
        RowIdentity,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        RowIdentity::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        PrincipalBinding,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PrincipalBinding::reference()
    }
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
}
impl ApplicationMutationIntent<Schema> for BumpRow {
    type Binding = BumpBinding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> <BumpBinding as ApplicationMutationBinding<Schema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(RowIdentity::reference(), self.row.clone())
    }
}
