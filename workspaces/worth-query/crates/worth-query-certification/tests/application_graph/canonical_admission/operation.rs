//! A real authored effect whose decision deliberately swallows field failures.
use super::{opaque::SwallowingValue, schema::*};
use std::cell::Cell;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub enum Reads {
    Root,
    Child,
    Both,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Input {
    pub reads: Reads,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "canonical-admission.input.v1" });
worth_query_operation!(pub Advance for AdmissionSchema, input InputBinding);
worth_query_operation_reads!(Advance => [DocumentIdentityField, DocumentValueField, CounterField, ChildLink]);
worth_query_operation_writes!(Advance => [CounterField]);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Written {
    pub counter: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Denied;
worth_query_structured_value_binding!(pub WrittenBinding for Written { identity: "canonical-admission.written.v1" });
worth_query_structured_value_binding!(pub DeniedBinding for Denied { identity: "canonical-admission.denied.v1" });

pub struct AdvanceBinding;
impl ApplicationMutationAdmissionBinding<AdmissionSchema> for AdvanceBinding {
    type Admission = ApplicationConventionalAdmission;
}
type Scope = ApplicationMutationFieldScope<
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
pub struct Decision {
    target: WorthQueryInvariantMutationTarget<AdmissionSchema, Document>,
    counter: u64,
}
impl ApplicationMutationBinding<AdmissionSchema> for AdvanceBinding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Written;
    type ResultBinding = WrittenBinding;
    type IdempotencyKey = u64;
    type Operation = Advance;
    type Decision = Decision;
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
    const IDENTITY: &'static str = "canonical-admission.advance-binding.v1";
    const HANDLER_IDENTITY: &'static str = "canonical-admission.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "canonical-admission.command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 2, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(1024),
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
pub struct Intent {
    pub input: Input,
}
impl ApplicationMutationIntent<AdmissionSchema> for Intent {
    type Binding = AdvanceBinding;
    fn input(&self) -> &Input {
        &self.input
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(DocumentIdentityField::reference(), "root".to_owned())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Observed {
    pub root: bool,
    pub child: bool,
    pub failures: u8,
    pub candidates: u8,
}
thread_local! { static OBSERVED: Cell<Observed> = Cell::default(); }
pub fn reset() {
    OBSERVED.set(Observed::default());
}
pub fn observed() -> Observed {
    OBSERVED.get()
}
fn record(value: Result<Option<SwallowingValue>, HandlerExecutionDenial>, root: bool) {
    let mut observation = OBSERVED.get();
    match value {
        Ok(Some(value)) => {
            assert_eq!(
                value.0,
                super::value(),
                "decode must preserve nondefault nested data"
            );
            if root {
                observation.root = true;
            } else {
                observation.child = true;
            }
        }
        _ => observation.failures += 1,
    }
    OBSERVED.set(observation);
}
pub struct Handler;
impl OperationHandler<AdmissionSchema, AdvanceBinding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, AdmissionSchema, AdvanceBinding>,
    ) -> HandlerResult<Decision, Denied> {
        let root =
            match reader.resolve_entity(DocumentIdentityField::reference(), "root".to_owned()) {
                Ok(root) => root,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
        let counter = match reader.field(&root, CounterField::reference()) {
            Ok(Some(counter)) => counter,
            Ok(None) => return HandlerResult::DomainDenied(Denied),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let target = match reader.mutation_target(&root) {
            Ok(target) => target,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let mut links = match reader.relations_from(ChildLink::reference(), &root) {
            Ok(links) => links,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        assert_eq!(links.len(), 1);
        let child = links.pop().unwrap().into_to();
        if !matches!(input.reads, Reads::Child) {
            record(reader.field(&root, DocumentValueField::reference()), true);
        }
        if !matches!(input.reads, Reads::Root) {
            record(reader.field(&child, DocumentValueField::reference()), false);
        }
        // Deliberately ignore any field failure: the native completion gate must veto this plan.
        HandlerResult::Completed(Decision {
            target,
            counter: counter + 1,
        })
    }
    fn candidate_requirements(&self, _: &Input, _: &Decision) -> ApplicationCandidateRequirements {
        AdvanceBinding::CANDIDATES
    }
    fn build_candidate(
        &self,
        _: &Input,
        decision: Decision,
        writer: &mut CandidateWriter<'_, AdmissionSchema, AdvanceBinding>,
    ) -> HandlerResult<Written, Denied> {
        let mut observation = OBSERVED.get();
        observation.candidates += 1;
        OBSERVED.set(observation);
        let target = match writer.projected_entity(&decision.target) {
            Ok(target) => target,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        match writer.write_field(&target, CounterField::reference(), decision.counter) {
            Ok(()) => HandlerResult::Completed(Written {
                counter: decision.counter,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
