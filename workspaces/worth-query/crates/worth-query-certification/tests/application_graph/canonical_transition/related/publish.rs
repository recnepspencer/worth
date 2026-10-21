//! An actual installed effect publication owns the live cause.
use super::cause::{Changed, ChangedBinding, ValueChanged};
use super::{
    producer::{Denied, DeniedBinding, Published, PublishedBinding},
    schema::*,
};
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    *,
};

worth_query_operation!(pub PublishValue for CanonicalSchema, input ChangedBinding);
worth_query_operation_emits!(PublishValue => [ValueChanged]);
pub struct PublishBinding;
type Scope = ApplicationMutationFieldScope<
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationMutationAdmissionBinding<CanonicalSchema> for PublishBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<CanonicalSchema> for PublishBinding {
    type Input = Changed;
    type InputBinding = ChangedBinding;
    type Result = Published;
    type ResultBinding = PublishedBinding;
    type IdempotencyKey = u64;
    type Operation = PublishValue;
    type Decision = ();
    type Denial = Denied;
    type DenialBinding = DeniedBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "canonical-related.publish-binding.v1";
    const HANDLER_IDENTITY: &'static str = "canonical-related.publish-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "canonical-related.publish-command.v1";
    const CANDIDATES: worth_query_decl::facade::application_operation::ApplicationCandidatePolicy =
        worth_query_decl::facade::application_operation::ApplicationCandidatePolicy::finite(
            ApplicationCandidateRequirements::fixed_shape(
                ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, 1),
                ApplicationCandidateResourceCeiling::representation_bytes(1_024),
            ),
        );
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        CanonicalSchema,
        Derived,
        DerivedFacts,
        DerivedIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DerivedIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        CanonicalSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

#[derive(Clone)]
pub struct Publish {
    pub changed: Changed,
}
impl ApplicationMutationIntent<CanonicalSchema> for Publish {
    type Binding = PublishBinding;
    fn input(&self) -> &Changed {
        &self.changed
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(
            DerivedIdentityField::reference(),
            self.changed.identity.clone(),
        )
    }
}
pub struct Handler;
impl OperationHandler<CanonicalSchema, PublishBinding> for Handler {
    fn decide(
        &self,
        _: &Changed,
        _: &mut DecisionReader<'_, '_, '_, CanonicalSchema, PublishBinding>,
    ) -> HandlerResult<(), Denied> {
        HandlerResult::Completed(())
    }
    fn candidate_requirements(&self, _: &Changed, _: &()) -> ApplicationCandidateRequirements {
        PublishBinding::CANDIDATES
            .finite_requirements()
            .expect("fixed candidate contract")
    }
    fn build_candidate(
        &self,
        input: &Changed,
        _: (),
        writer: &mut CandidateWriter<'_, CanonicalSchema, PublishBinding>,
    ) -> HandlerResult<Published, Denied> {
        match writer.emit(ValueChanged::reference(), input.clone()) {
            Ok(()) => HandlerResult::Completed(Published),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
