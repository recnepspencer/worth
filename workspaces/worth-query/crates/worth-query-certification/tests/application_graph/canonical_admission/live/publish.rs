//! An ordinary installed operation publishes the cause; it never calls a consumer.
use super::super::{
    operation::{Denied, DeniedBinding, Written, WrittenBinding},
    schema::*,
};
use super::cause::{Changed, ChangedBinding, ValueChanged};
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    *,
};

worth_query_operation!(pub PublishValue for AdmissionSchema, input ChangedBinding);
worth_query_operation_emits!(PublishValue => [ValueChanged]);
pub struct PublishBinding;
type Scope = ApplicationMutationFieldScope<
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationMutationAdmissionBinding<AdmissionSchema> for PublishBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<AdmissionSchema> for PublishBinding {
    type Input = Changed;
    type InputBinding = ChangedBinding;
    type Result = Written;
    type ResultBinding = WrittenBinding;
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
    const IDENTITY: &'static str = "canonical-admission.publish-binding.v1";
    const HANDLER_IDENTITY: &'static str = "canonical-admission.publish-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "canonical-admission.publish-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 0, 1),
            ApplicationCandidateResourceCeiling::representation_bytes(1_024),
        );
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        AdmissionSchema,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DocumentIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        AdmissionSchema,
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
impl ApplicationMutationIntent<AdmissionSchema> for Publish {
    type Binding = PublishBinding;
    fn input(&self) -> &Changed {
        &self.changed
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(
            DocumentIdentityField::reference(),
            self.changed.identity.clone(),
        )
    }
}
pub struct Handler;
impl OperationHandler<AdmissionSchema, PublishBinding> for Handler {
    fn decide(
        &self,
        _: &Changed,
        _: &mut DecisionReader<'_, '_, '_, AdmissionSchema, PublishBinding>,
    ) -> HandlerResult<(), Denied> {
        HandlerResult::Completed(())
    }
    fn candidate_requirements(&self, _: &Changed, _: &()) -> ApplicationCandidateRequirements {
        PublishBinding::CANDIDATES
    }
    fn build_candidate(
        &self,
        input: &Changed,
        _: (),
        writer: &mut CandidateWriter<'_, AdmissionSchema, PublishBinding>,
    ) -> HandlerResult<Written, Denied> {
        match writer.emit(ValueChanged::reference(), input.clone()) {
            Ok(()) => HandlerResult::Completed(Written { counter: 0 }),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
